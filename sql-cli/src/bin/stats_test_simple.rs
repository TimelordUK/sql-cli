use std::collections::{BTreeMap, HashSet};
use std::time::Instant;

fn main() {
    println!("Testing column statistics performance with simple data\n");

    // Test with 10k rows, 4 unique values
    let values: Vec<String> = (0..10000).map(|i| format!("value_{}", i % 4)).collect();

    println!(
        "Testing with {} rows, expecting 4 unique values",
        values.len()
    );

    // Test 1: HashSet for unique counting
    let start = Instant::now();
    let refs: Vec<&str> = values.iter().map(|s| s.as_str()).collect();
    let mut unique = HashSet::new();
    for value in &refs {
        unique.insert(*value);
    }
    println!(
        "HashSet unique count: {} in {:?}",
        unique.len(),
        start.elapsed()
    );

    // Test 2: BTreeMap for frequency
    let start = Instant::now();
    let mut freq_map: BTreeMap<&str, usize> = BTreeMap::new();
    for value in &refs {
        *freq_map.entry(*value).or_insert(0) += 1;
    }
    println!(
        "BTreeMap frequency: {} unique in {:?}",
        freq_map.len(),
        start.elapsed()
    );

    // Test 3: Convert to owned strings
    let start = Instant::now();
    let owned_freq: BTreeMap<String, usize> = freq_map
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    println!(
        "Convert to owned: {} entries in {:?}",
        owned_freq.len(),
        start.elapsed()
    );

    // Test 4: Full statistics calculation
    let start = Instant::now();
    let stats = calculate_full_stats(&refs);
    println!("\nFull stats calculation: {:?}", start.elapsed());
    println!("  Unique: {}", stats.unique_count);
    println!("  Total: {}", stats.total_count);
    println!(
        "  Frequency map entries: {}",
        stats.frequency_map.as_ref().map(|m| m.len()).unwrap_or(0)
    );

    // Test with actual string parsing for min/max
    let start = Instant::now();
    let mut min_str: Option<&str> = None;
    let mut max_str: Option<&str> = None;
    for value in &refs {
        match min_str {
            None => min_str = Some(value),
            Some(min) if value < &min => min_str = Some(value),
            _ => {}
        }
        match max_str {
            None => max_str = Some(value),
            Some(max) if value > &max => max_str = Some(value),
            _ => {}
        }
    }
    println!("\nMin/max calculation: {:?}", start.elapsed());
    println!("  Min: {:?}, Max: {:?}", min_str, max_str);
}

struct Stats {
    total_count: usize,
    unique_count: usize,
    frequency_map: Option<BTreeMap<String, usize>>,
}

fn calculate_full_stats(values: &[&str]) -> Stats {
    let mut unique = HashSet::new();

    for value in values {
        unique.insert(*value);
    }

    let frequency_map = if unique.len() <= 100 {
        let mut freq_map: BTreeMap<&str, usize> = BTreeMap::new();
        for value in values {
            *freq_map.entry(*value).or_insert(0) += 1;
        }
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
