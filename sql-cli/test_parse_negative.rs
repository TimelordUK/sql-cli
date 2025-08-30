use sql_cli::recursive_parser::Parser;

fn main() {
    let query = "SELECT * FROM test WHERE book.IndexOf(' ') = -1";
    let mut parser = Parser::new(query);
    let statement = parser.parse().expect("Failed to parse");
    
    println!("Parsed statement: {:#?}", statement);
    
    if let Some(where_clause) = statement.where_clause {
        println!("\nWHERE clause conditions: {:#?}", where_clause.conditions);
    }
}