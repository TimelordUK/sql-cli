use sql_cli::sql::recursive_parser::Parser;

fn main() {
    let sql = "SELECT * FROM users WHERE name LIKE '%e%'";
    let mut parser = Parser::new(sql);
    let result = parser.parse();

    match result {
        Ok(statement) => {
            println!("Parsed successfully!");
            println!("Statement: {:#?}", statement);
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
}
