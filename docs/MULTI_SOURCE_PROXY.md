# Multi-Source Data Proxy Architecture

## Overview
The SQL CLI now supports querying multiple data sources through a unified .NET Core proxy. This allows seamless querying across:
- SQL Server databases
- Public REST APIs
- CSV/JSON files
- Legacy trade data API

## Visual Indicators
The SQL CLI displays colored source indicators in the status bar:
- 📦 **CACHE** (cyan) - Data from local cache
- 📁 **FILE** (green) - Data from CSV/JSON files
- 🗄️ **SQL** (blue) - Data from SQL Server
- 🌐 **API** (yellow) - Data from external APIs
- 🔄 **PROXY** (magenta) - Generic proxy source

## Configuration

### API Server (TradeApi)
Edit `TradeApi/appsettings.json`:
```json
{
  "ConnectionStrings": {
    "SqlServer": "Server=localhost,1433;Database=TestDB;..."
  },
  "DataSources": {
    "FileDirectory": "../data"
  }
}
```

### Table Routing
The system automatically routes tables to appropriate data sources:
- `customers`, `small-customer` → FileDataSource (CSV files)
- `users`, `posts`, `todos` → PublicApiDataSource (JSONPlaceholder API)
- `client_mappings`, `instruments` → SqlServerDataSource
- `trade_deal` → Legacy TradeDataService

## Usage Examples

### Query CSV Files
```sql
SELECT * FROM customers WHERE Country = 'USA'
```
Status bar shows: 📁 **FILE**

### Query Public APIs
```sql
SELECT * FROM posts WHERE userId = 1
```
Status bar shows: 🌐 **API**

### Cache Management
```bash
# Save query results with named cache ID
:cache save trades_20240106

# Load from cache
:cache load trades_20240106
```
Status bar shows: 📦 **CACHE**

### Cross-Source Queries
The proxy handles queries across different sources transparently:
```sql
-- This queries a CSV file
SELECT * FROM customers

-- This queries a public API
SELECT * FROM users

-- This queries SQL Server (if configured)
SELECT * FROM client_mappings
```

## Architecture

### Data Source Interface
```csharp
public interface IDataSource
{
    string SourceName { get; }
    string[] SupportedTables { get; }
    Task<IQueryable<dynamic>> GetDataAsync(string tableName);
    Task<TableSchema> GetSchemaAsync(string tableName);
}
```

### Available Data Sources
1. **FileDataSource** - Reads CSV/JSON from local files
2. **SqlServerDataSource** - Connects to SQL Server databases
3. **PublicApiDataSource** - Fetches from REST APIs
4. **TradeDataService** - Legacy trade data generator

### Adding New Data Sources
1. Implement `IDataSource` interface
2. Register in `Program.cs`
3. Update `DataSourceRouter` with table mappings

## Running the System

### Start the API Server
```bash
cd TradeApi
dotnet run
```

### Use SQL CLI
```bash
cd sql-cli
cargo run --release
```

## Benefits
- **Unified Interface**: Query any data source with SQL
- **Visual Feedback**: Always know where your data comes from
- **Caching**: Save slow API results for fast local queries
- **Extensible**: Easy to add new data sources
- **Schema-Aware**: Autocomplete works across all sources