use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use crossterm::style::Stylize;
use serde_json::Value;

pub fn display_results(data: &[Value], fields: &[String]) {
    if data.is_empty() {
        println!("{}", "No results found.".yellow());
        return;
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Set headers
    let headers: Vec<Cell> = if fields.contains(&"*".to_string()) {
        // Get all fields from first record
        if let Some(first) = data.first() {
            if let Some(obj) = first.as_object() {
                obj.keys()
                    .map(|k| Cell::new(k).add_attribute(Attribute::Bold))
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        fields
            .iter()
            .map(|f| Cell::new(f).add_attribute(Attribute::Bold))
            .collect()
    };

    let field_names: Vec<String> = headers.iter().map(|h| h.content().to_string()).collect();
    table.set_header(headers);

    // Add rows
    for record in data {
        if let Some(obj) = record.as_object() {
            let row: Vec<String> = field_names
                .iter()
                .map(|field| match obj.get(field) {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Number(n)) => n.to_string(),
                    Some(Value::Bool(b)) => b.to_string(),
                    Some(Value::Null) => "NULL".to_string(),
                    Some(v) => v.to_string(),
                    None => "".to_string(),
                })
                .collect();
            table.add_row(row);
        }
    }

    println!("{table}");
    println!("\n{}", format!("{} rows returned", data.len()).green());
}

pub fn export_to_csv(
    data: &[Value],
    fields: &[String],
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(filename)?;

    // Write headers
    let headers: Vec<String> = if fields.contains(&"*".to_string()) {
        if let Some(first) = data.first() {
            if let Some(obj) = first.as_object() {
                obj.keys().map(|k| k.clone()).collect()
            } else {
                return Err("Invalid data format".into());
            }
        } else {
            return Ok(());
        }
    } else {
        fields.to_vec()
    };

    wtr.write_record(&headers)?;

    // Write data
    for record in data {
        if let Some(obj) = record.as_object() {
            let row: Vec<String> = headers
                .iter()
                .map(|field| match obj.get(field) {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Number(n)) => n.to_string(),
                    Some(Value::Bool(b)) => b.to_string(),
                    Some(Value::Null) => "".to_string(),
                    Some(v) => v.to_string(),
                    None => "".to_string(),
                })
                .collect();
            wtr.write_record(&row)?;
        }
    }

    wtr.flush()?;
    println!("{}", format!("Results exported to {}", filename).green());
    Ok(())
}
