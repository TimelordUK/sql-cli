use anyhow::Result;

use crate::api_client::QueryResponse;
use crate::csv_datasource::CsvApiClient;
use crate::data::datatable::DataTable;

/// Trait for executing SQL queries against data sources
pub trait QueryExecutor {
    /// Execute a SQL query and return results
    fn execute(&self, query: &str) -> Result<QueryResponse>;

    /// Check if this executor can handle the given query
    fn can_handle(&self, query: &str) -> bool;

    /// Get row count without executing a query
    fn row_count(&self) -> usize;

    /// Get column count without executing a query
    fn column_count(&self) -> usize;
}

/// Direct DataTable query executor (for simple SELECT * queries)
pub struct DataTableExecutor {
    datatable: std::sync::Arc<DataTable>,
    table_name: String,
}

impl DataTableExecutor {
    pub fn new(datatable: std::sync::Arc<DataTable>, table_name: String) -> Self {
        Self {
            datatable,
            table_name,
        }
    }
}

impl QueryExecutor for DataTableExecutor {
    fn execute(&self, query: &str) -> Result<QueryResponse> {
        // For now, only handle SELECT * FROM table
        let upper_query = query.trim().to_uppercase();
        if !self.can_handle(query) {
            return Err(anyhow::anyhow!(
                "DataTableExecutor can only handle simple SELECT * queries"
            ));
        }

        // Return a response that references the DataTable directly
        // In the future, this will return a DataView
        Ok(QueryResponse {
            data: vec![], // Empty for now - TUI will use DataTable directly
            count: self.datatable.row_count(),
            query: crate::api_client::QueryInfo {
                select: vec!["*".to_string()],
                where_clause: None,
                order_by: None,
            },
            source: Some("datatable".to_string()),
            table: Some(self.table_name.clone()),
            cached: Some(false),
        })
    }

    fn can_handle(&self, query: &str) -> bool {
        let upper_query = query.trim().to_uppercase();
        upper_query.starts_with("SELECT *")
            && !upper_query.contains(" WHERE ")
            && !upper_query.contains(" ORDER BY ")
            && !upper_query.contains(" LIMIT ")
            && !upper_query.contains(" GROUP BY ")
    }

    fn row_count(&self) -> usize {
        self.datatable.row_count()
    }

    fn column_count(&self) -> usize {
        self.datatable.column_count()
    }
}

/// CSV API Client query executor (for complex queries with WHERE, ORDER BY, etc.)
pub struct CsvClientExecutor {
    csv_client: CsvApiClient,
    table_name: String,
}

impl CsvClientExecutor {
    pub fn new(csv_client: CsvApiClient, table_name: String) -> Self {
        Self {
            csv_client,
            table_name,
        }
    }

    pub fn from_csv(path: &str, table_name: &str, case_insensitive: bool) -> Result<Self> {
        let mut csv_client = CsvApiClient::new();
        csv_client.set_case_insensitive(case_insensitive);
        csv_client.load_csv(path, table_name)?;
        Ok(Self {
            csv_client,
            table_name: table_name.to_string(),
        })
    }

    pub fn from_json(path: &str, table_name: &str, case_insensitive: bool) -> Result<Self> {
        let mut csv_client = CsvApiClient::new();
        csv_client.set_case_insensitive(case_insensitive);
        csv_client.load_json(path, table_name)?;
        Ok(Self {
            csv_client,
            table_name: table_name.to_string(),
        })
    }
}

impl QueryExecutor for CsvClientExecutor {
    fn execute(&self, query: &str) -> Result<QueryResponse> {
        let result = self.csv_client.query_csv(query)?;
        Ok(QueryResponse {
            data: result.data,
            count: result.count,
            query: crate::api_client::QueryInfo {
                select: result.query.select,
                where_clause: result.query.where_clause,
                order_by: result.query.order_by,
            },
            source: Some("csv_client".to_string()),
            table: Some(self.table_name.clone()),
            cached: Some(false),
        })
    }

    fn can_handle(&self, _query: &str) -> bool {
        // CSV client can handle all queries
        true
    }

    fn row_count(&self) -> usize {
        // This is approximate - CSV client doesn't expose exact count
        0
    }

    fn column_count(&self) -> usize {
        // This is approximate - CSV client doesn't expose exact count
        0
    }
}

/// Composite query executor that tries multiple executors in order
pub struct CompositeQueryExecutor {
    executors: Vec<Box<dyn QueryExecutor>>,
}

impl CompositeQueryExecutor {
    pub fn new() -> Self {
        Self {
            executors: Vec::new(),
        }
    }

    pub fn add_executor(&mut self, executor: Box<dyn QueryExecutor>) {
        self.executors.push(executor);
    }
}

impl QueryExecutor for CompositeQueryExecutor {
    fn execute(&self, query: &str) -> Result<QueryResponse> {
        // Try each executor in order
        for executor in &self.executors {
            if executor.can_handle(query) {
                return executor.execute(query);
            }
        }
        Err(anyhow::anyhow!("No executor can handle query: {}", query))
    }

    fn can_handle(&self, query: &str) -> bool {
        self.executors.iter().any(|e| e.can_handle(query))
    }

    fn row_count(&self) -> usize {
        // Return the row count from the first executor
        self.executors.first().map(|e| e.row_count()).unwrap_or(0)
    }

    fn column_count(&self) -> usize {
        // Return the column count from the first executor
        self.executors
            .first()
            .map(|e| e.column_count())
            .unwrap_or(0)
    }
}
