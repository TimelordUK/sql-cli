use sql_cli::sql::recursive_parser::Parser;

fn main() {
    let query = "SELECT * FROM test WHERE country NOT IN ('CA')";
    println!("Parsing query: {}", query);
    
    let mut parser = Parser::new(query);
    match parser.parse() {
        Ok(statement) => {
            println!("Parsed statement: {:#?}", statement);
            if let Some(where_clause) = statement.where_clause {
                println!("WHERE conditions: {:#?}", where_clause.conditions);
            }
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
}