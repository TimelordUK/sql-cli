use crate::sql::parser::ast::{SelectStatement, SqlExpression, WhereClause};
use std::collections::{HashMap, HashSet};

/// Represents a unit of work in the query execution pipeline
#[derive(Debug, Clone)]
pub struct WorkUnit {
    /// Unique identifier for this work unit
    pub id: String,

    /// Type of work this unit performs
    pub work_type: WorkUnitType,

    /// SQL expression or statement to execute
    pub expression: WorkUnitExpression,

    /// Dependencies - IDs of work units that must complete before this one
    pub dependencies: Vec<String>,

    /// Whether this unit can be executed in parallel with siblings
    pub parallelizable: bool,

    /// Cost estimate for query optimization
    pub cost_estimate: Option<f64>,
}

/// Types of work units in the execution pipeline
#[derive(Debug, Clone, PartialEq)]
pub enum WorkUnitType {
    /// Base table scan
    TableScan,

    /// CTE definition
    CTE,

    /// Filter operation (WHERE clause)
    Filter,

    /// Aggregation (GROUP BY)
    Aggregate,

    /// Sorting (ORDER BY)
    Sort,

    /// Join operation
    Join,

    /// Window function computation
    Window,

    /// Expression evaluation
    Expression,

    /// Final projection (SELECT)
    Projection,
}

/// Expression or statement in a work unit
#[derive(Debug, Clone)]
pub enum WorkUnitExpression {
    /// Full SELECT statement (for CTEs)
    Select(SelectStatement),

    /// Single expression (for filters, projections)
    Expression(SqlExpression),

    /// WHERE clause
    WhereClause(WhereClause),

    /// Table name for base scans
    TableName(String),

    /// Custom operation
    Custom(String),
}

/// Complete query execution plan
#[derive(Debug)]
pub struct QueryPlan {
    /// All work units in the plan
    pub units: Vec<WorkUnit>,

    /// Dependency graph for determining execution order
    pub dependency_graph: DependencyGraph,

    /// Estimated total cost
    pub total_cost: Option<f64>,

    /// Original query for reference
    pub original_query: String,

    /// Metadata about the plan
    pub metadata: PlanMetadata,
}

impl QueryPlan {
    /// Create a new empty query plan
    pub fn new(original_query: String) -> Self {
        QueryPlan {
            units: Vec::new(),
            dependency_graph: DependencyGraph::new(),
            original_query,
            total_cost: None,
            metadata: PlanMetadata::default(),
        }
    }

    /// Add a work unit to the plan
    pub fn add_unit(&mut self, unit: WorkUnit) {
        // Add to dependency graph
        for dep in &unit.dependencies {
            self.dependency_graph.add_edge(dep.clone(), unit.id.clone());
        }

        // Store the unit
        self.units.push(unit);
    }

    /// Get execution order respecting dependencies
    pub fn get_execution_order(&self) -> Result<Vec<String>, String> {
        self.dependency_graph.topological_sort()
    }

    /// Get units that can be executed in parallel
    pub fn get_parallel_groups(&self) -> Vec<Vec<String>> {
        self.dependency_graph.get_parallel_groups()
    }

    /// Optimize the plan (placeholder for future optimization logic)
    pub fn optimize(&mut self) -> Result<(), String> {
        // Future: implement cost-based optimization
        // - Reorder operations when possible
        // - Push down filters
        // - Merge adjacent operations
        Ok(())
    }

    /// Generate a human-readable representation of the plan
    pub fn explain(&self) -> String {
        let mut output = String::new();
        output.push_str("Query Execution Plan:\n");
        output.push_str("====================\n\n");

        // Show execution order
        match self.get_execution_order() {
            Ok(order) => {
                output.push_str("Execution Order:\n");
                for (i, unit_id) in order.iter().enumerate() {
                    if let Some(unit) = self.units.iter().find(|u| u.id == *unit_id) {
                        output.push_str(&format!(
                            "  {}. {} ({:?})\n",
                            i + 1,
                            unit.id,
                            unit.work_type
                        ));

                        if !unit.dependencies.is_empty() {
                            output.push_str(&format!(
                                "     Dependencies: {}\n",
                                unit.dependencies.join(", ")
                            ));
                        }

                        if unit.parallelizable {
                            output.push_str("     [Parallelizable]\n");
                        }
                    }
                }
            }
            Err(e) => {
                output.push_str(&format!("Error determining execution order: {}\n", e));
            }
        }

        // Show parallel groups
        output.push_str("\nParallel Execution Groups:\n");
        for (i, group) in self.get_parallel_groups().iter().enumerate() {
            output.push_str(&format!("  Group {}: {}\n", i + 1, group.join(", ")));
        }

        output
    }
}

/// Metadata about the query plan
#[derive(Debug, Default)]
pub struct PlanMetadata {
    /// Whether CTEs were lifted from WHERE clause
    pub has_lifted_expressions: bool,

    /// Number of parallel execution opportunities
    pub parallel_opportunities: usize,

    /// Estimated row count
    pub estimated_rows: Option<usize>,

    /// Planning time in milliseconds
    pub planning_time_ms: Option<u64>,
}

/// Dependency graph for work units
#[derive(Debug)]
pub struct DependencyGraph {
    /// Adjacency list representation
    edges: HashMap<String, HashSet<String>>,

    /// All nodes in the graph
    nodes: HashSet<String>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        DependencyGraph {
            edges: HashMap::new(),
            nodes: HashSet::new(),
        }
    }

    /// Add an edge from source to target (source must complete before target)
    pub fn add_edge(&mut self, source: String, target: String) {
        self.nodes.insert(source.clone());
        self.nodes.insert(target.clone());

        self.edges
            .entry(source)
            .or_insert_with(HashSet::new)
            .insert(target);
    }

    /// Perform topological sort to get valid execution order
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut result = Vec::new();

        // Initialize in-degrees
        for node in &self.nodes {
            in_degree.insert(node.clone(), 0);
        }

        // Calculate in-degrees
        for (_, targets) in &self.edges {
            for target in targets {
                *in_degree.get_mut(target).unwrap() += 1;
            }
        }

        // Find nodes with no dependencies
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(node, _)| node.clone())
            .collect();

        // Process nodes
        while !queue.is_empty() {
            let node = queue.remove(0);
            result.push(node.clone());

            // Update in-degrees of dependent nodes
            if let Some(targets) = self.edges.get(&node) {
                for target in targets {
                    let degree = in_degree.get_mut(target).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(target.clone());
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != self.nodes.len() {
            return Err("Dependency cycle detected in query plan".to_string());
        }

        Ok(result)
    }

    /// Get groups of units that can be executed in parallel
    pub fn get_parallel_groups(&self) -> Vec<Vec<String>> {
        let mut groups = Vec::new();
        let mut remaining = self.nodes.clone();
        let mut completed = HashSet::new();

        while !remaining.is_empty() {
            let mut current_group = Vec::new();

            // Find all nodes whose dependencies are satisfied
            for node in &remaining {
                let deps_satisfied = self
                    .edges
                    .iter()
                    .filter(|(_, targets)| targets.contains(node))
                    .all(|(source, _)| completed.contains(source));

                if deps_satisfied {
                    current_group.push(node.clone());
                }
            }

            // If no nodes can be executed, we have a problem
            if current_group.is_empty() && !remaining.is_empty() {
                // This shouldn't happen if topological sort succeeds
                break;
            }

            // Mark these nodes as completed
            for node in &current_group {
                completed.insert(node.clone());
                remaining.remove(node);
            }

            if !current_group.is_empty() {
                groups.push(current_group);
            }
        }

        groups
    }

    /// Check if the graph has cycles
    pub fn has_cycles(&self) -> bool {
        self.topological_sort().is_err()
    }
}

/// Query analyzer that builds execution plans
pub struct QueryAnalyzer {
    /// Counter for generating unique work unit IDs
    unit_counter: usize,
}

impl QueryAnalyzer {
    /// Create a new query analyzer
    pub fn new() -> Self {
        QueryAnalyzer { unit_counter: 0 }
    }

    /// Generate a unique ID for a work unit
    fn next_unit_id(&mut self, prefix: &str) -> String {
        self.unit_counter += 1;
        format!("{}_{}", prefix, self.unit_counter)
    }

    /// Analyze a SELECT statement and build an execution plan
    pub fn analyze(&mut self, stmt: &SelectStatement, query: String) -> Result<QueryPlan, String> {
        let mut plan = QueryPlan::new(query);

        // Phase 1: Add base table scan
        let table_unit = WorkUnit {
            id: self.next_unit_id("scan"),
            work_type: WorkUnitType::TableScan,
            expression: WorkUnitExpression::TableName(
                stmt.from_table
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            ),
            dependencies: Vec::new(),
            parallelizable: false,
            cost_estimate: None,
        };
        let table_id = table_unit.id.clone();
        plan.add_unit(table_unit);

        // Phase 2: Analyze WHERE clause for liftable expressions
        let mut filter_id = None;
        if let Some(ref where_clause) = stmt.where_clause {
            // TODO: Implement expression lifting logic here
            // For now, just add as a simple filter
            let filter_unit = WorkUnit {
                id: self.next_unit_id("filter"),
                work_type: WorkUnitType::Filter,
                expression: WorkUnitExpression::WhereClause(where_clause.clone()),
                dependencies: vec![table_id.clone()],
                parallelizable: false,
                cost_estimate: None,
            };
            filter_id = Some(filter_unit.id.clone());
            plan.add_unit(filter_unit);
        }

        // Phase 3: Handle GROUP BY
        let mut group_id = None;
        if stmt.group_by.as_ref().map_or(false, |g| !g.is_empty()) {
            let dependencies = vec![filter_id.clone().unwrap_or(table_id.clone())];
            let group_unit = WorkUnit {
                id: self.next_unit_id("group"),
                work_type: WorkUnitType::Aggregate,
                expression: WorkUnitExpression::Custom("GROUP BY".to_string()),
                dependencies,
                parallelizable: false,
                cost_estimate: None,
            };
            group_id = Some(group_unit.id.clone());
            plan.add_unit(group_unit);
        }

        // Phase 4: Handle ORDER BY
        let mut sort_id = None;
        if stmt.order_by.as_ref().map_or(false, |o| !o.is_empty()) {
            let dependencies = vec![group_id
                .clone()
                .or(filter_id.clone())
                .unwrap_or(table_id.clone())];
            let sort_unit = WorkUnit {
                id: self.next_unit_id("sort"),
                work_type: WorkUnitType::Sort,
                expression: WorkUnitExpression::Custom("ORDER BY".to_string()),
                dependencies,
                parallelizable: false,
                cost_estimate: None,
            };
            sort_id = Some(sort_unit.id.clone());
            plan.add_unit(sort_unit);
        }

        // Phase 5: Final projection
        let dependencies = vec![sort_id.or(group_id).or(filter_id).unwrap_or(table_id)];
        let projection_unit = WorkUnit {
            id: self.next_unit_id("project"),
            work_type: WorkUnitType::Projection,
            expression: WorkUnitExpression::Custom("SELECT".to_string()),
            dependencies,
            parallelizable: false,
            cost_estimate: None,
        };
        plan.add_unit(projection_unit);

        // Optimize the plan
        plan.optimize()?;

        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();

        // Create a simple DAG
        graph.add_edge("A".to_string(), "B".to_string());
        graph.add_edge("A".to_string(), "C".to_string());
        graph.add_edge("B".to_string(), "D".to_string());
        graph.add_edge("C".to_string(), "D".to_string());

        // Test topological sort
        let order = graph.topological_sort().unwrap();
        assert_eq!(order.len(), 4);

        // A should come before B and C
        let a_pos = order.iter().position(|x| x == "A").unwrap();
        let b_pos = order.iter().position(|x| x == "B").unwrap();
        let c_pos = order.iter().position(|x| x == "C").unwrap();
        let d_pos = order.iter().position(|x| x == "D").unwrap();

        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
        assert!(b_pos < d_pos);
        assert!(c_pos < d_pos);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();

        // Create a cycle
        graph.add_edge("A".to_string(), "B".to_string());
        graph.add_edge("B".to_string(), "C".to_string());
        graph.add_edge("C".to_string(), "A".to_string());

        assert!(graph.has_cycles());
    }

    #[test]
    fn test_parallel_groups() {
        let mut graph = DependencyGraph::new();

        // Create independent branches
        graph.add_edge("A".to_string(), "B".to_string());
        graph.add_edge("A".to_string(), "C".to_string());
        graph.add_edge("B".to_string(), "D".to_string());
        graph.add_edge("C".to_string(), "E".to_string());

        let groups = graph.get_parallel_groups();

        // A should be alone, B and C can be parallel, D and E can be parallel
        assert!(groups.len() >= 3);
    }
}
