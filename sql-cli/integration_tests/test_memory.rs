#!/usr/bin/env rust-script
//! Test memory usage at different stages of data loading
//!
//! ```cargo
//! [dependencies]
//! csv = "1.3"
//! serde_json = "1.0"
//! ```

use std::fs::File;
use std::mem;
use csv::Reader;
use serde_json::{json, Value};

fn get_memory_usage() -> usize {
    // This is a rough estimate - in production use a proper memory profiler
    let mut status = String::new();
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|line| line.starts_with("VmRSS:"))
        .map(|line| {
            line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

fn main() {
    println!("Memory Usage Investigation");
    println!("==========================\n");
    
    let initial_memory = get_memory_usage();
    println!("Initial memory: {} KB", initial_memory);
    
    // Test 1: Load CSV into Vec<Vec<String>> (simplest format)
    println!("\n1. Loading CSV as Vec<Vec<String>>...");
    let file = File::open("trades_10k.csv").expect("Cannot open trades_10k.csv");
    let mut reader = Reader::from_reader(file);
    
    let headers: Vec<String> = reader.headers()
        .unwrap()
        .iter()
        .map(|h| h.to_string())
        .collect();
    
    let mut simple_data: Vec<Vec<String>> = Vec::new();
    for result in reader.records() {
        let record = result.unwrap();
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        simple_data.push(row);
    }
    
    let after_simple = get_memory_usage();
    let simple_size = after_simple - initial_memory;
    println!("After loading {} rows as strings: {} KB", simple_data.len(), after_simple);
    println!("Memory used: {} KB ({} bytes per row)", 
             simple_size, 
             (simple_size * 1024) / simple_data.len());
    
    // Test 2: Convert to JSON (like CsvDataSource does)
    println!("\n2. Converting to JSON objects...");
    let mut json_data: Vec<Value> = Vec::new();
    for row in &simple_data {
        let mut obj = serde_json::Map::new();
        for (i, value) in row.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                obj.insert(header.clone(), Value::String(value.clone()));
            }
        }
        json_data.push(Value::Object(obj));
    }
    
    let after_json = get_memory_usage();
    let json_size = after_json - after_simple;
    println!("After converting to JSON: {} KB", after_json);
    println!("Additional memory for JSON: {} KB ({} bytes per row)",
             json_size,
             (json_size * 1024) / json_data.len());
    
    // Test 3: Clone the JSON data (simulating QueryResponse)
    println!("\n3. Cloning JSON (simulating QueryResponse)...");
    let json_clone = json_data.clone();
    
    let after_clone = get_memory_usage();
    let clone_size = after_clone - after_json;
    println!("After cloning JSON: {} KB", after_clone);
    println!("Additional memory for clone: {} KB", clone_size);
    
    // Test 4: Create string table for rendering (like filtered_data)
    println!("\n4. Creating string table for rendering...");
    let mut string_table: Vec<Vec<String>> = Vec::new();
    for value in &json_data {
        if let Some(obj) = value.as_object() {
            let row: Vec<String> = headers.iter()
                .map(|h| obj.get(h)
                    .map(|v| v.to_string())
                    .unwrap_or_default())
                .collect();
            string_table.push(row);
        }
    }
    
    let after_strings = get_memory_usage();
    let strings_size = after_strings - after_clone;
    println!("After creating string table: {} KB", after_strings);
    println!("Additional memory for strings: {} KB", strings_size);
    
    // Summary
    println!("\n=== SUMMARY ===");
    println!("Total memory used: {} KB", after_strings - initial_memory);
    println!("Per row: {} bytes", ((after_strings - initial_memory) * 1024) / simple_data.len());
    println!("\nBreakdown:");
    println!("  Simple strings: {} KB ({:.1}%)", simple_size, (simple_size as f64 / (after_strings - initial_memory) as f64) * 100.0);
    println!("  JSON objects:   {} KB ({:.1}%)", json_size, (json_size as f64 / (after_strings - initial_memory) as f64) * 100.0);
    println!("  JSON clone:     {} KB ({:.1}%)", clone_size, (clone_size as f64 / (after_strings - initial_memory) as f64) * 100.0);
    println!("  String table:   {} KB ({:.1}%)", strings_size, (strings_size as f64 / (after_strings - initial_memory) as f64) * 100.0);
    
    // Keep data alive so it's not freed
    println!("\nData sizes in memory:");
    println!("  simple_data: {} rows", simple_data.len());
    println!("  json_data: {} rows", json_data.len());
    println!("  json_clone: {} rows", json_clone.len());
    println!("  string_table: {} rows", string_table.len());
}