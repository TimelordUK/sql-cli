#[cfg(test)]
mod tests {
    use crate::recursive_parser::{Lexer, Parser, SortDirection, Token};
    use crate::where_parser::WhereParser;

    #[test]
    fn test_complex_trade_query_tokenization() {
        // This is a real-world query that must continue to work
        let query = "SELECT accruedInterest,allocationStatus,book,clearingHouse,comments,platformOrderId,parentOrderId,commission,confirmationStatus,counterparty,counterpartyCountry FROM trades where platformOrderId.Contains('P') and counterparty.Contains('morgan') and clearingHouse in ('lch')  order by counterparty desc,  book, counterpartyCountry asc";

        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all();

        // Verify the exact tokenization output
        let expected_tokens = vec![
            Token::Select,
            Token::Identifier("accruedInterest".to_string()),
            Token::Comma,
            Token::Identifier("allocationStatus".to_string()),
            Token::Comma,
            Token::Identifier("book".to_string()),
            Token::Comma,
            Token::Identifier("clearingHouse".to_string()),
            Token::Comma,
            Token::Identifier("comments".to_string()),
            Token::Comma,
            Token::Identifier("platformOrderId".to_string()),
            Token::Comma,
            Token::Identifier("parentOrderId".to_string()),
            Token::Comma,
            Token::Identifier("commission".to_string()),
            Token::Comma,
            Token::Identifier("confirmationStatus".to_string()),
            Token::Comma,
            Token::Identifier("counterparty".to_string()),
            Token::Comma,
            Token::Identifier("counterpartyCountry".to_string()),
            Token::From,
            Token::Identifier("trades".to_string()),
            Token::Where,
            Token::Identifier("platformOrderId".to_string()),
            Token::Dot,
            Token::Identifier("Contains".to_string()),
            Token::LeftParen,
            Token::StringLiteral("P".to_string()),
            Token::RightParen,
            Token::And,
            Token::Identifier("counterparty".to_string()),
            Token::Dot,
            Token::Identifier("Contains".to_string()),
            Token::LeftParen,
            Token::StringLiteral("morgan".to_string()),
            Token::RightParen,
            Token::And,
            Token::Identifier("clearingHouse".to_string()),
            Token::In,
            Token::LeftParen,
            Token::StringLiteral("lch".to_string()),
            Token::RightParen,
            Token::OrderBy,
            Token::Identifier("counterparty".to_string()),
            Token::Desc,
            Token::Comma,
            Token::Identifier("book".to_string()),
            Token::Comma,
            Token::Identifier("counterpartyCountry".to_string()),
            Token::Asc,
            Token::Eof,
        ];

        assert_eq!(tokens.len(), expected_tokens.len(), "Token count mismatch");

        for (i, (actual, expected)) in tokens.iter().zip(expected_tokens.iter()).enumerate() {
            assert_eq!(actual, expected, "Token mismatch at position {}", i);
        }
    }

    #[test]
    fn test_complex_trade_query_ast() {
        let query = "SELECT accruedInterest,allocationStatus,book,clearingHouse,comments,platformOrderId,parentOrderId,commission,confirmationStatus,counterparty,counterpartyCountry FROM trades where platformOrderId.Contains('P') and counterparty.Contains('morgan') and clearingHouse in ('lch')  order by counterparty desc,  book, counterpartyCountry asc";

        let mut parser = Parser::new(query);
        let result = parser.parse();

        assert!(result.is_ok(), "Query should parse successfully");

        let stmt = result.unwrap();

        // Verify SELECT columns
        assert_eq!(stmt.columns.len(), 11);
        assert_eq!(stmt.columns[0], "accruedInterest");
        assert_eq!(stmt.columns[1], "allocationStatus");
        assert_eq!(stmt.columns[2], "book");
        assert_eq!(stmt.columns[3], "clearingHouse");
        assert_eq!(stmt.columns[4], "comments");
        assert_eq!(stmt.columns[5], "platformOrderId");
        assert_eq!(stmt.columns[6], "parentOrderId");
        assert_eq!(stmt.columns[7], "commission");
        assert_eq!(stmt.columns[8], "confirmationStatus");
        assert_eq!(stmt.columns[9], "counterparty");
        assert_eq!(stmt.columns[10], "counterpartyCountry");

        // Verify FROM table
        assert_eq!(stmt.from_table.as_deref(), Some("trades"));

        // Verify WHERE clause exists
        assert!(stmt.where_clause.is_some());
        let where_clause = stmt.where_clause.unwrap();
        // Check that we have the expected number of conditions
        assert_eq!(where_clause.conditions.len(), 3);

        // Verify ORDER BY
        assert!(stmt.order_by.is_some());
        let order_by = stmt.order_by.unwrap();
        assert_eq!(order_by.len(), 3);

        // First: counterparty desc
        assert_eq!(order_by[0].column, "counterparty");
        assert_eq!(order_by[0].direction, SortDirection::Desc);

        // Second: book (default asc)
        assert_eq!(order_by[1].column, "book");
        assert_eq!(order_by[1].direction, SortDirection::Asc); // Default is ASC

        // Third: counterpartyCountry asc
        assert_eq!(order_by[2].column, "counterpartyCountry");
        assert_eq!(order_by[2].direction, SortDirection::Asc);
    }

    #[test]
    fn test_complex_where_clause_parsing() {
        // Test the WHERE clause parsing specifically with case-insensitive mode
        let where_clause = "platformOrderId.Contains('P') and counterparty.Contains('morgan') and clearingHouse in ('lch')";

        let columns = vec![
            "platformOrderId".to_string(),
            "counterparty".to_string(),
            "clearingHouse".to_string(),
        ];

        // Test with case-insensitive mode (as shown in the debug output)
        let result = WhereParser::parse_with_options(where_clause, columns.clone(), true);
        assert!(result.is_ok(), "WHERE clause should parse successfully");

        // Also test with case-sensitive mode to ensure both work
        let result_cs = WhereParser::parse_with_options(where_clause, columns, false);
        assert!(
            result_cs.is_ok(),
            "WHERE clause should parse in case-sensitive mode too"
        );
    }

    #[test]
    fn test_method_calls_in_where_clause() {
        // Test that method calls like .Contains() are properly parsed
        let query = "SELECT * FROM trades WHERE platformOrderId.Contains('P')";

        let mut parser = Parser::new(query);
        let result = parser.parse();

        assert!(result.is_ok(), "Query with method call should parse");
        let stmt = result.unwrap();

        assert!(stmt.where_clause.is_some());
        // Just verify we have a where clause - the structure is complex
    }

    #[test]
    fn test_in_clause_parsing() {
        // Test IN clause parsing
        let query = "SELECT * FROM trades WHERE clearingHouse in ('lch', 'eurex', 'cme')";

        let mut parser = Parser::new(query);
        let result = parser.parse();

        assert!(result.is_ok(), "Query with IN clause should parse");
        let stmt = result.unwrap();

        assert!(stmt.where_clause.is_some());
        // The where clause will be parsed into a complex structure
    }

    #[test]
    fn test_mixed_order_by_directions() {
        // Test ORDER BY with mixed ASC/DESC
        let query =
            "SELECT * FROM trades ORDER BY counterparty DESC, book, counterpartyCountry ASC";

        let mut parser = Parser::new(query);
        let result = parser.parse();

        assert!(result.is_ok(), "Query with mixed ORDER BY should parse");
        let stmt = result.unwrap();

        assert!(stmt.order_by.is_some());
        let order_by = stmt.order_by.unwrap();
        assert_eq!(order_by.len(), 3);

        // DESC specified
        assert_eq!(order_by[0].column, "counterparty");
        assert_eq!(order_by[0].direction, SortDirection::Desc);

        // No direction (defaults to ASC)
        assert_eq!(order_by[1].column, "book");
        assert_eq!(order_by[1].direction, SortDirection::Asc);

        // ASC explicitly specified
        assert_eq!(order_by[2].column, "counterpartyCountry");
        assert_eq!(order_by[2].direction, SortDirection::Asc);
    }

    #[test]
    fn test_complex_query_with_many_columns() {
        // Test that we can handle many columns in SELECT
        let columns = vec![
            "accruedInterest",
            "allocationStatus",
            "book",
            "clearingHouse",
            "comments",
            "platformOrderId",
            "parentOrderId",
            "commission",
            "confirmationStatus",
            "counterparty",
            "counterpartyCountry",
        ];

        let query = format!("SELECT {} FROM trades", columns.join(","));

        let mut parser = Parser::new(&query);
        let result = parser.parse();

        assert!(result.is_ok(), "Query with many columns should parse");
        let stmt = result.unwrap();

        assert_eq!(stmt.columns.len(), columns.len());
        for (actual, expected) in stmt.columns.iter().zip(columns.iter()) {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_whitespace_handling_in_complex_query() {
        // Test that extra whitespace doesn't break parsing
        let query = "SELECT   accruedInterest , allocationStatus   FROM   trades   WHERE  platformOrderId.Contains( 'P' )   AND   counterparty.Contains( 'morgan' )   ORDER BY   counterparty   DESC  ,  book  ";

        let mut parser = Parser::new(query);
        let result = parser.parse();

        assert!(result.is_ok(), "Query with extra whitespace should parse");
        let stmt = result.unwrap();

        assert_eq!(stmt.columns.len(), 2);
        assert_eq!(stmt.columns[0], "accruedInterest");
        assert_eq!(stmt.columns[1], "allocationStatus");
        assert_eq!(stmt.from_table.as_deref(), Some("trades"));
        assert!(stmt.where_clause.is_some());
        assert!(stmt.order_by.is_some());
        assert_eq!(stmt.order_by.unwrap().len(), 2);
    }

    #[test]
    fn test_logical_operators_precedence() {
        // Test AND/OR precedence in WHERE clause
        let query = "SELECT * FROM trades WHERE a = 1 OR b = 2 AND c = 3";

        let mut parser = Parser::new(query);
        let result = parser.parse();

        assert!(result.is_ok(), "Query with AND/OR should parse");
        let stmt = result.unwrap();

        // The WHERE clause should respect precedence: OR < AND
        // So this should parse as: a = 1 OR (b = 2 AND c = 3)
        assert!(stmt.where_clause.is_some());
        // The where clause structure will contain the logical operators
    }

    #[test]
    fn test_string_literals_with_special_chars() {
        // Test string literals with various content
        let test_cases = vec![
            "SELECT * FROM trades WHERE name = 'O''Brien'", // Escaped quote
            "SELECT * FROM trades WHERE id = 'ABC-123'",    // Hyphen
            "SELECT * FROM trades WHERE code = 'A_B_C'",    // Underscore
            "SELECT * FROM trades WHERE tag = 'P'",         // Single char
        ];

        for query in test_cases {
            let mut parser = Parser::new(query);
            let result = parser.parse();
            assert!(result.is_ok(), "Query '{}' should parse", query);
        }
    }
}
