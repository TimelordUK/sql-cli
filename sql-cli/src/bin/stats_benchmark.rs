use std::collections::{BTreeMap, HashSet};
use std::time::Instant;

fn main() {
    println!("Column Statistics Benchmark\n");

    // Generate test data - 10k values with varying uniqueness
    let test_cases = vec![
        ("Few unique (10)", generate_test_data(10000, 10)),
        ("Medium unique (100)", generate_test_data(10000, 100)),
        ("Many unique (1000)", generate_test_data(10000, 1000)),
        ("All unique (10000)", generate_test_data(10000, 10000)),
    ];

    for (name, data) in test_cases {
        println!("Test case: {} - {} total values", name, data.len());

        // Convert to &str for the optimized version
        let str_refs: Vec<&str> = data.iter().map(|s| s.as_str()).collect();

        // Benchmark original approach (with cloning)
        let start = Instant::now();
        let stats1 = calculate_stats_with_cloning(&data);
        let duration1 = start.elapsed();
        println!("  With cloning: {:?}", duration1);

        // Benchmark optimized approach (without cloning)
        let start = Instant::now();
        let stats2 = calculate_stats_without_cloning(&str_refs);
        let duration2 = start.elapsed();
        println!("  Without cloning: {:?}", duration2);

        // Calculate speedup
        let speedup = duration1.as_secs_f64() / duration2.as_secs_f64();
        println!("  Speedup: {:.2}x faster", speedup);

        // Verify results are the same
        assert_eq!(stats1.unique_count, stats2.unique_count);
        assert_eq!(stats1.total_count, stats2.total_count);
        println!("  Unique values: {}", stats1.unique_count);
        println!();
    }
}

fn generate_test_data(total: usize, unique: usize) -> Vec<String> {
    let mut data = Vec::with_capacity(total);
    let unique = unique.min(total);

    // Generate unique strings
    let unique_strings: Vec<String> = (0..unique)
        .map(|i| format!("value_{:05}_with_some_longer_text_to_simulate_real_data", i))
        .collect();

    // Fill the rest by repeating
    for i in 0..total {
        data.push(unique_strings[i % unique].clone());
    }

    data
}

#[derive(Debug)]
struct Stats {
    total_count: usize,
    unique_count: usize,
    frequency_map: Option<BTreeMap<String, usize>>,
}

// Original approach with cloning
fn calculate_stats_with_cloning(values: &[String]) -> Stats {
    let mut unique = HashSet::new();
    let mut frequency_map = BTreeMap::new();

    for value in values {
        // Clone for unique set
        unique.insert(value.clone());

        // Clone for frequency map
        *frequency_map.entry(value.clone()).or_insert(0) += 1;
    }

    Stats {
        total_count: values.len(),
        unique_count: unique.len(),
        frequency_map: if frequency_map.len() <= 100 {
            Some(frequency_map)
        } else {
            None
        },
    }
}

// Optimized approach without cloning
fn calculate_stats_without_cloning(values: &[&str]) -> Stats {
    let mut unique = HashSet::new();

    for value in values {
        // No clone for unique set
        unique.insert(*value);
    }

    // Only build frequency map if not too many unique values
    let frequency_map = if unique.len() <= 100 {
        let mut freq_map: BTreeMap<&str, usize> = BTreeMap::new();
        for value in values {
            *freq_map.entry(*value).or_insert(0) += 1;
        }
        // Only convert to owned at the end
        Some(
            freq_map
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    } else {
        None
    };

    Stats {
        total_count: values.len(),
        unique_count: unique.len(),
        frequency_map,
    }
}
