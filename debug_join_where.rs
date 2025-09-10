use sql_cli::sql::recursive_parser::Parser;

fn main() {
    let query = "SELECT * FROM users JOIN orders ON users.id = orders.user_id WHERE orders.total > 100";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    
    match result {
        Ok(stmt) => {
            println!("Parse successful!");
            println!("Joins: {:?}", stmt.joins.len());
            println!("Has WHERE: {}", stmt.where_clause.is_some());
        }
        Err(e) => println!("Parse error: {}", e),
    }
}