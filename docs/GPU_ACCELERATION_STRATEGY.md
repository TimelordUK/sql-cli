# GPU Acceleration Strategy for Rust TUI SQL Engine

## Overview
This document outlines a comprehensive strategy for leveraging GPU acceleration in a Rust-based TUI SQL engine, specifically targeting NVIDIA RTX 4060 Ti (16GB VRAM) on WSL2 with CUDA support.

## Architecture Overview

### Hybrid CPU/GPU Query Execution Pipeline
```
Query → AST Parser → Predicate Splitter → GPU Filter → CPU Processing → TUI Display
         ↓              ↓                    ↓            ↓
      Parse Tree   GPU/CPU Split      Reduced Dataset  Final Results
```

## GPU-Accelerated Operations

### Perfect for GPU (10-100x speedup potential)
- **String Pattern Matching**: `LIKE`, `StartsWith`, `Contains`
- **Range Filters**: `BETWEEN`, `>`, `<`, `>=`, `<=`
- **Set Membership**: `IN` clauses
- **Aggregations**: `SUM`, `COUNT`, `AVG`, `GROUP BY`
- **Sorting**: Radix sort for large datasets
- **Column Scans**: Full table scans across columns

### Keep on CPU
- Complex regex patterns
- JOINs with small lookup tables
- Window functions
- TUI rendering logic
- Small dataset operations (< 10K rows)

## Implementation Strategy

### 1. AST-Based GPU Offloading

```rust
use std::boxed::Box;

#[derive(Debug, Clone)]
enum QueryNode {
    And(Box<QueryNode>, Box<QueryNode>),
    Or(Box<QueryNode>, Box<QueryNode>),
    StartsWith { column: String, value: String },
    Contains { column: String, value: String },
    GreaterThan { column: String, value: f64 },
    Between { column: String, min: f64, max: f64 },
    In { column: String, values: Vec<String> },
    // Complex operations stay on CPU
    Regex { column: String, pattern: String },
    Join { left: String, right: String, condition: Box<QueryNode> },
}

impl QueryNode {
    fn can_offload_to_gpu(&self) -> bool {
        match self {
            // These parallelize perfectly on GPU
            StartsWith{..} | Contains{..} | GreaterThan{..} | 
            Between{..} | In{..} => true,
            
            // Recursive check for logical operators
            And(left, right) | Or(left, right) => 
                left.can_offload_to_gpu() && right.can_offload_to_gpu(),
            
            // Keep complex operations on CPU
            Regex{..} | Join{..} => false,
        }
    }
    
    fn estimated_gpu_speedup(&self, row_count: usize) -> f32 {
        if row_count < 10_000 {
            return 0.8; // GPU overhead not worth it
        }
        
        match self {
            StartsWith{..} | Contains{..} => (row_count as f32 / 10_000.0) * 10.0,
            GreaterThan{..} | Between{..} => (row_count as f32 / 10_000.0) * 15.0,
            In{..} => (row_count as f32 / 10_000.0) * 20.0,
            And(l, r) | Or(l, r) => {
                (l.estimated_gpu_speedup(row_count) + 
                 r.estimated_gpu_speedup(row_count)) / 2.0
            },
            _ => 1.0,
        }
    }
}
```

### 2. Predicate Splitting Algorithm

```rust
/// Split query into GPU-friendly and CPU-only predicates
fn split_predicates(ast: QueryNode) -> (Option<QueryNode>, Option<QueryNode>) {
    match ast {
        QueryNode::And(left, right) => {
            let (gpu_left, cpu_left) = split_predicates(*left);
            let (gpu_right, cpu_right) = split_predicates(*right);
            
            // Combine GPU predicates with AND
            let gpu = combine_optional(gpu_left, gpu_right, |l, r| {
                QueryNode::And(Box::new(l), Box::new(r))
            });
            
            let cpu = combine_optional(cpu_left, cpu_right, |l, r| {
                QueryNode::And(Box::new(l), Box::new(r))
            });
            
            (gpu, cpu)
        },
        
        QueryNode::Or(left, right) => {
            // OR requires both sides on same processor
            if left.can_offload_to_gpu() && right.can_offload_to_gpu() {
                (Some(ast), None)
            } else {
                (None, Some(ast))
            }
        },
        
        node if node.can_offload_to_gpu() => (Some(node), None),
        node => (None, Some(node))
    }
}

fn combine_optional<F>(
    left: Option<QueryNode>, 
    right: Option<QueryNode>, 
    combiner: F
) -> Option<QueryNode> 
where F: FnOnce(QueryNode, QueryNode) -> QueryNode 
{
    match (left, right) {
        (Some(l), Some(r)) => Some(combiner(l, r)),
        (Some(l), None) => Some(l),
        (None, Some(r)) => Some(r),
        (None, None) => None,
    }
}
```

### 3. GPU Table Structure

```rust
use wgpu::{Device, Buffer, Queue};

pub struct GpuTable {
    device: Device,
    queue: Queue,
    
    // Column-oriented storage for GPU efficiency
    columns: Vec<GpuColumn>,
    row_count: usize,
    
    // Pre-allocated buffers for results
    result_buffer: Buffer,
    staging_buffer: Buffer,
}

pub struct GpuColumn {
    name: String,
    data_type: DataType,
    data_buffer: Buffer,
    null_bitmap: Buffer,  // For nullable columns
    string_offsets: Option<Buffer>,  // For variable-length strings
}

impl GpuTable {
    /// Upload table to GPU memory
    pub async fn from_datatable(
        device: &Device,
        queue: &Queue,
        table: &DataTable
    ) -> Result<Self, GpuError> {
        let columns = table.columns.iter()
            .map(|col| Self::upload_column(device, queue, col))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(Self {
            device: device.clone(),
            queue: queue.clone(),
            columns,
            row_count: table.row_count(),
            result_buffer: Self::allocate_result_buffer(device, table.row_count()),
            staging_buffer: Self::allocate_staging_buffer(device),
        })
    }
    
    /// Execute GPU predicates and return matching row indices
    pub async fn filter(&self, predicates: &QueryNode) -> Vec<u32> {
        let shader = self.compile_predicate_shader(predicates);
        let pipeline = self.create_compute_pipeline(&shader);
        
        // Dispatch compute shader with 1 thread per row
        let workgroups = (self.row_count + 255) / 256;
        
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());
            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroups as u32, 1, 1);
        }
        
        // Copy results to staging buffer
        encoder.copy_buffer_to_buffer(
            &self.result_buffer, 0,
            &self.staging_buffer, 0,
            (self.row_count * 4) as u64
        );
        
        self.queue.submit(Some(encoder.finish()));
        
        // Read back results
        self.read_results().await
    }
}
```

### 4. Compute Shader Example (WGSL)

```wgsl
// StartsWith predicate shader
@group(0) @binding(0) var<storage, read> column_data: array<u32>;
@group(0) @binding(1) var<storage, read> string_offsets: array<u32>;
@group(0) @binding(2) var<storage, read> string_pool: array<u32>;
@group(0) @binding(3) var<uniform> prefix: array<u32, 16>; // Up to 64 chars
@group(0) @binding(4) var<storage, read_write> results: array<atomic<u32>>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let row_idx = global_id.x;
    if (row_idx >= arrayLength(&column_data)) {
        return;
    }
    
    // Get string for this row
    let str_start = string_offsets[row_idx];
    let str_end = string_offsets[row_idx + 1];
    let str_len = str_end - str_start;
    
    // Check if string starts with prefix
    var matches = true;
    for (var i = 0u; i < 16u; i = i + 1u) {
        if (prefix[i] == 0u) { break; } // End of prefix
        if (i >= str_len) { 
            matches = false;
            break;
        }
        if (string_pool[str_start + i] != prefix[i]) {
            matches = false;
            break;
        }
    }
    
    // Set bit in result bitmap
    if (matches) {
        let word_idx = row_idx / 32u;
        let bit_idx = row_idx % 32u;
        atomicOr(&results[word_idx], 1u << bit_idx);
    }
}
```

### 5. Execution Pipeline

```rust
pub struct HybridQueryExecutor {
    cpu_executor: CpuQueryExecutor,
    gpu_executor: Option<GpuQueryExecutor>,
    gpu_threshold: usize,  // Min rows to use GPU
}

impl HybridQueryExecutor {
    pub async fn execute(&self, query: &str, table: &DataTable) -> QueryResult {
        // 1. Parse query to AST
        let ast = parse_query(query)?;
        
        // 2. Check if GPU acceleration is worth it
        if table.row_count() < self.gpu_threshold || self.gpu_executor.is_none() {
            return self.cpu_executor.execute(&ast, table);
        }
        
        // 3. Split predicates into GPU and CPU parts
        let (gpu_predicates, cpu_predicates) = split_predicates(ast.where_clause);
        
        // 4. Execute GPU predicates first (if any)
        let filtered_indices = if let Some(gpu_preds) = gpu_predicates {
            let gpu_table = GpuTable::from_datatable(
                &self.gpu_executor.device,
                &self.gpu_executor.queue,
                table
            ).await?;
            
            gpu_table.filter(&gpu_preds).await
        } else {
            // All rows if no GPU predicates
            (0..table.row_count() as u32).collect()
        };
        
        info!("GPU filtered {} rows to {}", 
              table.row_count(), 
              filtered_indices.len());
        
        // 5. Transfer only matching rows to CPU
        let reduced_table = table.select_rows(&filtered_indices);
        
        // 6. Apply CPU predicates on reduced dataset
        if let Some(cpu_preds) = cpu_predicates {
            self.cpu_executor.execute_with_predicates(
                &ast.select_clause,
                &cpu_preds,
                &reduced_table
            )
        } else {
            QueryResult::from_table(reduced_table)
        }
    }
}
```

## Rust GPU Libraries Comparison

### 1. **wgpu** (Recommended for portability)
```toml
[dependencies]
wgpu = "0.19"
```
- **Pros**: Pure Rust, cross-platform, WebGPU standard
- **Cons**: Lower-level, more boilerplate
- **Use for**: Maximum portability

### 2. **CUDA via cust** (Maximum performance)
```toml
[dependencies]
cust = "0.3"
```
- **Pros**: Direct CUDA access, maximum performance
- **Cons**: NVIDIA-only, requires CUDA toolkit
- **Use for**: NVIDIA-specific optimizations

### 3. **ArrayFire** (High-level operations)
```toml
[dependencies]
arrayfire = "3.8"
```
- **Pros**: High-level operations, SQL-like functions built-in
- **Cons**: Large dependency, less control
- **Use for**: Rapid prototyping

### 4. **Candle** (ML framework repurposed)
```toml
[dependencies]
candle = "0.3"
```
- **Pros**: Tensor operations work well for tables
- **Cons**: Designed for ML, not databases
- **Use for**: If already using for ML features

## Performance Considerations

### Memory Transfer Optimization
```rust
// Batch multiple small queries
pub struct QueryBatcher {
    pending_queries: Vec<QueryNode>,
    flush_threshold: Duration,
}

// Keep hot data on GPU
pub struct GpuCache {
    cached_tables: HashMap<String, GpuTable>,
    max_cache_size: usize,
    lru: LinkedList<String>,
}

// Minimize transfer with bitmap results
pub struct BitmapResult {
    matching_rows: BitVec,  // 1 bit per row
    total_matches: u32,
}
```

### Benchmarking Framework
```rust
#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn benchmark_string_search(c: &mut Criterion) {
        let table = generate_test_table(1_000_000);
        
        c.bench_function("cpu_startswith", |b| {
            b.iter(|| {
                cpu_filter_startswith(black_box(&table), "ABC")
            })
        });
        
        c.bench_function("gpu_startswith", |b| {
            b.iter(|| {
                gpu_filter_startswith(black_box(&table), "ABC")
            })
        });
    }
}
```

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
- [ ] Set up wgpu in project
- [ ] Create basic GPU table structure
- [ ] Implement simple numeric filters (>, <, =)
- [ ] Benchmark vs CPU implementation

### Phase 2: String Operations (Week 3-4)
- [ ] Implement GPU string storage
- [ ] Add StartsWith, Contains predicates
- [ ] Optimize string memory layout
- [ ] Add IN clause support

### Phase 3: Query Optimization (Week 5-6)
- [ ] AST analyzer for GPU offloading
- [ ] Predicate splitting algorithm
- [ ] Cost-based optimizer
- [ ] Query batching system

### Phase 4: Production Features (Week 7-8)
- [ ] GPU memory management
- [ ] Error handling and fallback
- [ ] Performance monitoring
- [ ] Configuration and tuning

## Configuration

```toml
# config.toml
[gpu]
enabled = true
device_index = 0  # For multi-GPU systems
min_rows_threshold = 50000
max_gpu_memory = "8GB"
shader_cache_dir = "~/.cache/sql-cli/shaders"

[gpu.operations]
string_search = true
numeric_filters = true
aggregations = true
sorting = false  # Experimental

[gpu.fallback]
on_error = "cpu"  # or "fail"
timeout_ms = 5000
```

## Monitoring and Debugging

```rust
pub struct GpuMetrics {
    pub kernel_time_ms: f32,
    pub transfer_time_ms: f32,
    pub memory_used_mb: usize,
    pub speedup_factor: f32,
}

impl GpuQueryExecutor {
    pub fn with_metrics<F>(&self, f: F) -> (QueryResult, GpuMetrics) 
    where F: FnOnce() -> QueryResult {
        let start = Instant::now();
        let gpu_mem_before = self.get_memory_usage();
        
        let result = f();
        
        let metrics = GpuMetrics {
            kernel_time_ms: self.last_kernel_time(),
            transfer_time_ms: self.last_transfer_time(),
            memory_used_mb: self.get_memory_usage() - gpu_mem_before,
            speedup_factor: self.calculate_speedup(),
        };
        
        (result, metrics)
    }
}
```

## Example: Complete Query Flow

```rust
// User query: 
// "SELECT * FROM trades 
//  WHERE symbol.StartsWith('AAP') 
//    AND price > 100.0 
//    AND timestamp BETWEEN '2024-01-01' AND '2024-12-31'
//    AND client_name ~ 'Corp.*Ltd'"  // Regex stays on CPU

// 1. Parse to AST
let ast = QueryNode::And(
    Box::new(QueryNode::And(
        Box::new(QueryNode::StartsWith { 
            column: "symbol".into(), 
            value: "AAP".into() 
        }),
        Box::new(QueryNode::GreaterThan { 
            column: "price".into(), 
            value: 100.0 
        })
    )),
    Box::new(QueryNode::And(
        Box::new(QueryNode::Between { 
            column: "timestamp".into(),
            min: parse_date("2024-01-01"),
            max: parse_date("2024-12-31")
        }),
        Box::new(QueryNode::Regex { 
            column: "client_name".into(),
            pattern: "Corp.*Ltd".into()
        })
    ))
);

// 2. Split predicates
// GPU: symbol.StartsWith('AAP') AND price > 100.0 AND timestamp BETWEEN ...
// CPU: client_name ~ 'Corp.*Ltd'

// 3. GPU execution reduces 1M rows to 5K rows

// 4. CPU applies regex on 5K rows instead of 1M

// 5. Result: 100x speedup for initial filtering
```

## Future Enhancements

### Advanced GPU Operations
- **Parallel JOINs**: Hash joins on GPU for large tables
- **GPU-accelerated indexes**: B-tree operations in parallel
- **Stream processing**: Real-time filtering as data arrives
- **Multi-GPU support**: Partition large tables across GPUs

### Machine Learning Integration
```rust
// Use GPU for both SQL and ML inference
pub struct SmartQueryOptimizer {
    gpu_context: GpuContext,
    ml_model: CandleModel,  // Query cost predictor
    
    pub fn optimize(&self, query: &QueryNode) -> ExecutionPlan {
        // Use ML to predict optimal execution strategy
        let features = self.extract_query_features(query);
        let predicted_costs = self.ml_model.predict(&features);
        
        self.generate_plan(query, predicted_costs)
    }
}
```

## Conclusion

GPU acceleration can provide 10-100x speedup for specific operations in your SQL engine, particularly for:
- Large dataset filtering (> 100K rows)
- String pattern matching at scale
- Aggregations and grouping
- Parallel sorting

The key is intelligent query planning that:
1. Identifies GPU-friendly operations
2. Minimizes data transfer overhead
3. Falls back gracefully to CPU when appropriate
4. Maintains TUI responsiveness during processing

With your RTX 4060 Ti's 16GB VRAM, you can keep entire multi-GB tables in GPU memory, making this architecture extremely powerful for interactive data exploration.