use crate::recursive_parser::{Lexer, Token};
use crate::where_ast::{ComparisonOp, WhereExpr, WhereValue};
use anyhow::{anyhow, Result};
use chrono::{Datelike, Local};

pub struct WhereParser {
    tokens: Vec<Token>,
    current: usize,
}

impl WhereParser {
    pub fn parse(where_clause: &str) -> Result<WhereExpr> {
        let mut lexer = Lexer::new(where_clause);
        let mut tokens = Vec::new();

        loop {
            let token = lexer.next_token();
            if matches!(token, Token::Eof) {
                break;
            }
            tokens.push(token);
        }

        let mut parser = WhereParser { tokens, current: 0 };
        parser.parse_or_expr()
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.current);
        self.current += 1;
        token
    }

    fn expect_identifier(&mut self) -> Result<String> {
        match self.advance() {
            Some(Token::Identifier(name)) => Ok(name.clone()),
            Some(Token::QuotedIdentifier(name)) => Ok(name.clone()), // Handle quoted column names
            _ => Err(anyhow!("Expected identifier")),
        }
    }

    fn parse_value(&mut self) -> Result<WhereValue> {
        match self.current_token() {
            Some(Token::StringLiteral(_)) => {
                if let Some(Token::StringLiteral(s)) = self.advance() {
                    Ok(WhereValue::String(s.clone()))
                } else {
                    unreachable!()
                }
            }
            Some(Token::QuotedIdentifier(_)) => {
                // Handle double-quoted strings as string literals in value context
                if let Some(Token::QuotedIdentifier(s)) = self.advance() {
                    Ok(WhereValue::String(s.clone()))
                } else {
                    unreachable!()
                }
            }
            Some(Token::NumberLiteral(_)) => {
                if let Some(Token::NumberLiteral(n)) = self.advance() {
                    Ok(WhereValue::Number(n.parse::<f64>().unwrap_or(0.0)))
                } else {
                    unreachable!()
                }
            }
            Some(Token::Null) => {
                self.advance();
                Ok(WhereValue::Null)
            }
            Some(Token::DateTime) => {
                self.advance(); // consume DateTime
                self.expect_token(Token::LeftParen)?;

                // Check if empty (today at midnight)
                if matches!(self.current_token(), Some(Token::RightParen)) {
                    self.advance(); // consume )
                    let today = Local::now();
                    let date_str = format!(
                        "{:04}-{:02}-{:02} 00:00:00",
                        today.year(),
                        today.month(),
                        today.day()
                    );
                    Ok(WhereValue::String(date_str))
                } else {
                    // Parse year, month, day, etc.
                    let year = self.parse_number_value()? as i32;
                    self.expect_token(Token::Comma)?;
                    let month = self.parse_number_value()? as u32;
                    self.expect_token(Token::Comma)?;
                    let day = self.parse_number_value()? as u32;

                    let mut hour = 0u32;
                    let mut minute = 0u32;
                    let mut second = 0u32;

                    // Optional time components
                    if matches!(self.current_token(), Some(Token::Comma)) {
                        self.advance(); // consume comma
                        hour = self.parse_number_value()? as u32;

                        if matches!(self.current_token(), Some(Token::Comma)) {
                            self.advance(); // consume comma
                            minute = self.parse_number_value()? as u32;

                            if matches!(self.current_token(), Some(Token::Comma)) {
                                self.advance(); // consume comma
                                second = self.parse_number_value()? as u32;
                            }
                        }
                    }

                    self.expect_token(Token::RightParen)?;

                    let date_str = format!(
                        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                        year, month, day, hour, minute, second
                    );
                    Ok(WhereValue::String(date_str))
                }
            }
            _ => Err(anyhow!("Expected value")),
        }
    }

    // Parse OR expressions (lowest precedence)
    fn parse_or_expr(&mut self) -> Result<WhereExpr> {
        let mut left = self.parse_and_expr()?;

        while let Some(Token::Or) = self.current_token() {
            self.advance(); // consume OR
            let right = self.parse_and_expr()?;
            left = WhereExpr::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    // Parse AND expressions
    fn parse_and_expr(&mut self) -> Result<WhereExpr> {
        let mut left = self.parse_not_expr()?;

        while let Some(Token::And) = self.current_token() {
            self.advance(); // consume AND
            let right = self.parse_not_expr()?;
            left = WhereExpr::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    // Parse NOT expressions
    fn parse_not_expr(&mut self) -> Result<WhereExpr> {
        if let Some(Token::Not) = self.current_token() {
            self.advance(); // consume NOT
            let expr = self.parse_comparison_expr()?;
            Ok(WhereExpr::Not(Box::new(expr)))
        } else {
            self.parse_comparison_expr()
        }
    }

    // Parse comparison expressions
    fn parse_comparison_expr(&mut self) -> Result<WhereExpr> {
        // Check for parentheses
        if let Some(Token::LeftParen) = self.current_token() {
            self.advance(); // consume (
            let expr = self.parse_or_expr()?;
            match self.advance() {
                Some(Token::RightParen) => Ok(expr),
                _ => Err(anyhow!("Expected closing parenthesis")),
            }
        } else {
            self.parse_primary_expr()
        }
    }

    // Parse primary expressions (columns, methods, comparisons)
    fn parse_primary_expr(&mut self) -> Result<WhereExpr> {
        let column = self.expect_identifier()?;

        // Check for method calls
        if let Some(Token::Dot) = self.current_token() {
            self.advance(); // consume .
            let method = self.expect_identifier()?;

            match method.as_str() {
                "Contains" => {
                    self.expect_token(Token::LeftParen)?;
                    let value = self.parse_string_value()?;
                    self.expect_token(Token::RightParen)?;
                    Ok(WhereExpr::Contains(column, value))
                }
                "StartsWith" => {
                    self.expect_token(Token::LeftParen)?;
                    let value = self.parse_string_value()?;
                    self.expect_token(Token::RightParen)?;
                    Ok(WhereExpr::StartsWith(column, value))
                }
                "EndsWith" => {
                    self.expect_token(Token::LeftParen)?;
                    let value = self.parse_string_value()?;
                    self.expect_token(Token::RightParen)?;
                    Ok(WhereExpr::EndsWith(column, value))
                }
                "Length" => {
                    self.expect_token(Token::LeftParen)?;
                    self.expect_token(Token::RightParen)?;

                    // Parse comparison operator
                    let op = self.parse_comparison_op()?;
                    let value = self.parse_number_value()?;
                    Ok(WhereExpr::Length(column, op, value as i64))
                }
                "ToLower" => {
                    self.expect_token(Token::LeftParen)?;
                    self.expect_token(Token::RightParen)?;

                    // Parse comparison operator
                    let op = self.parse_comparison_op()?;
                    let value = self.parse_string_value()?;
                    Ok(WhereExpr::ToLower(column, op, value))
                }
                "ToUpper" => {
                    self.expect_token(Token::LeftParen)?;
                    self.expect_token(Token::RightParen)?;

                    // Parse comparison operator
                    let op = self.parse_comparison_op()?;
                    let value = self.parse_string_value()?;
                    Ok(WhereExpr::ToUpper(column, op, value))
                }
                _ => Err(anyhow!("Unknown method: {}", method)),
            }
        } else {
            // Check for operators
            match self.current_token() {
                Some(Token::Equal) => {
                    self.advance();
                    let value = self.parse_value()?;
                    Ok(WhereExpr::Equal(column, value))
                }
                Some(Token::NotEqual) => {
                    self.advance();
                    let value = self.parse_value()?;
                    Ok(WhereExpr::NotEqual(column, value))
                }
                Some(Token::GreaterThan) => {
                    self.advance();
                    let value = self.parse_value()?;
                    Ok(WhereExpr::GreaterThan(column, value))
                }
                Some(Token::GreaterThanOrEqual) => {
                    self.advance();
                    let value = self.parse_value()?;
                    Ok(WhereExpr::GreaterThanOrEqual(column, value))
                }
                Some(Token::LessThan) => {
                    self.advance();
                    let value = self.parse_value()?;
                    Ok(WhereExpr::LessThan(column, value))
                }
                Some(Token::LessThanOrEqual) => {
                    self.advance();
                    let value = self.parse_value()?;
                    Ok(WhereExpr::LessThanOrEqual(column, value))
                }
                Some(Token::Between) => {
                    self.advance();
                    let lower = self.parse_value()?;
                    self.expect_token(Token::And)?;
                    let upper = self.parse_value()?;
                    Ok(WhereExpr::Between(column, lower, upper))
                }
                Some(Token::In) => {
                    self.advance();
                    self.expect_token(Token::LeftParen)?;
                    let values = self.parse_value_list()?;
                    self.expect_token(Token::RightParen)?;
                    Ok(WhereExpr::In(column, values))
                }
                Some(Token::Not) if matches!(self.peek_token(), Some(Token::In)) => {
                    self.advance(); // consume NOT
                    self.advance(); // consume IN
                    self.expect_token(Token::LeftParen)?;
                    let values = self.parse_value_list()?;
                    self.expect_token(Token::RightParen)?;
                    Ok(WhereExpr::NotIn(column, values))
                }
                Some(Token::Like) => {
                    self.advance();
                    let pattern = self.parse_string_value()?;
                    Ok(WhereExpr::Like(column, pattern))
                }
                Some(Token::Is) => {
                    self.advance();
                    match self.current_token() {
                        Some(Token::Null) => {
                            self.advance();
                            Ok(WhereExpr::IsNull(column))
                        }
                        Some(Token::Not) if matches!(self.peek_token(), Some(Token::Null)) => {
                            self.advance(); // consume NOT
                            self.advance(); // consume NULL
                            Ok(WhereExpr::IsNotNull(column))
                        }
                        _ => Err(anyhow!("Expected NULL or NOT NULL after IS")),
                    }
                }
                _ => Err(anyhow!("Expected operator after column")),
            }
        }
    }

    fn parse_comparison_op(&mut self) -> Result<ComparisonOp> {
        match self.advance() {
            Some(Token::Equal) => Ok(ComparisonOp::Equal),
            Some(Token::NotEqual) => Ok(ComparisonOp::NotEqual),
            Some(Token::GreaterThan) => Ok(ComparisonOp::GreaterThan),
            Some(Token::GreaterThanOrEqual) => Ok(ComparisonOp::GreaterThanOrEqual),
            Some(Token::LessThan) => Ok(ComparisonOp::LessThan),
            Some(Token::LessThanOrEqual) => Ok(ComparisonOp::LessThanOrEqual),
            _ => Err(anyhow!("Expected comparison operator")),
        }
    }

    fn parse_string_value(&mut self) -> Result<String> {
        match self.advance() {
            Some(Token::StringLiteral(s)) => Ok(s.clone()),
            Some(Token::QuotedIdentifier(s)) => Ok(s.clone()), // Handle double-quoted strings
            _ => Err(anyhow!("Expected string literal")),
        }
    }

    fn parse_number_value(&mut self) -> Result<f64> {
        match self.advance() {
            Some(Token::NumberLiteral(n)) => {
                n.parse::<f64>().map_err(|_| anyhow!("Invalid number"))
            }
            _ => Err(anyhow!("Expected number literal")),
        }
    }

    fn parse_value_list(&mut self) -> Result<Vec<WhereValue>> {
        let mut values = vec![self.parse_value()?];

        while let Some(Token::Comma) = self.current_token() {
            self.advance(); // consume comma
            values.push(self.parse_value()?);
        }

        Ok(values)
    }

    fn expect_token(&mut self, expected: Token) -> Result<()> {
        match self.advance() {
            Some(token) if std::mem::discriminant(token) == std::mem::discriminant(&expected) => {
                Ok(())
            }
            Some(token) => Err(anyhow!("Expected {:?}, got {:?}", expected, token)),
            None => Err(anyhow!("Unexpected end of input")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_comparison() {
        let expr = WhereParser::parse("price > 100").unwrap();
        match expr {
            WhereExpr::GreaterThan(col, val) => {
                assert_eq!(col, "price");
                assert_eq!(val, WhereValue::Number(100.0));
            }
            _ => panic!("Wrong expression type"),
        }
    }

    #[test]
    fn test_and_expression() {
        let expr = WhereParser::parse("price > 100 AND category = \"Electronics\"").unwrap();
        match expr {
            WhereExpr::And(left, right) => {
                match left.as_ref() {
                    WhereExpr::GreaterThan(col, val) => {
                        assert_eq!(col, "price");
                        assert_eq!(val, &WhereValue::Number(100.0));
                    }
                    _ => panic!("Wrong left expression"),
                }
                match right.as_ref() {
                    WhereExpr::Equal(col, val) => {
                        assert_eq!(col, "category");
                        assert_eq!(val, &WhereValue::String("Electronics".to_string()));
                    }
                    _ => panic!("Wrong right expression"),
                }
            }
            _ => panic!("Wrong expression type"),
        }
    }

    #[test]
    fn test_between_with_and() {
        let expr = WhereParser::parse(
            "category = \"Electronics\" AND price BETWEEN 100 AND 500 AND quantity > 0",
        )
        .unwrap();
        // This should parse as: (category = "Electronics") AND (price BETWEEN 100 AND 500) AND (quantity > 0)
        match expr {
            WhereExpr::And(left, right) => {
                // The parser is left-associative, so it's ((category = "Electronics") AND (price BETWEEN 100 AND 500)) AND (quantity > 0)
                match left.as_ref() {
                    WhereExpr::And(ll, lr) => {
                        match ll.as_ref() {
                            WhereExpr::Equal(col, val) => {
                                assert_eq!(col, "category");
                                assert_eq!(val, &WhereValue::String("Electronics".to_string()));
                            }
                            _ => panic!("Wrong leftmost expression"),
                        }
                        match lr.as_ref() {
                            WhereExpr::Between(col, lower, upper) => {
                                assert_eq!(col, "price");
                                assert_eq!(lower, &WhereValue::Number(100.0));
                                assert_eq!(upper, &WhereValue::Number(500.0));
                            }
                            _ => panic!("Wrong middle expression"),
                        }
                    }
                    _ => panic!("Wrong left structure"),
                }
                match right.as_ref() {
                    WhereExpr::GreaterThan(col, val) => {
                        assert_eq!(col, "quantity");
                        assert_eq!(val, &WhereValue::Number(0.0));
                    }
                    _ => panic!("Wrong right expression"),
                }
            }
            _ => panic!("Wrong expression type"),
        }
    }

    #[test]
    fn test_parentheses_precedence() {
        // Test that parentheses override default precedence
        // Default: a = 1 OR b = 2 AND c = 3 -> a = 1 OR (b = 2 AND c = 3)
        let expr1 = WhereParser::parse("a = 1 OR b = 2 AND c = 3").unwrap();
        match expr1 {
            WhereExpr::Or(left, right) => {
                // Left should be a = 1
                match left.as_ref() {
                    WhereExpr::Equal(col, val) => {
                        assert_eq!(col, "a");
                        assert_eq!(val, &WhereValue::Number(1.0));
                    }
                    _ => panic!("Wrong left expression"),
                }
                // Right should be (b = 2 AND c = 3)
                match right.as_ref() {
                    WhereExpr::And(l, r) => {
                        match l.as_ref() {
                            WhereExpr::Equal(col, val) => {
                                assert_eq!(col, "b");
                                assert_eq!(val, &WhereValue::Number(2.0));
                            }
                            _ => panic!("Wrong AND left"),
                        }
                        match r.as_ref() {
                            WhereExpr::Equal(col, val) => {
                                assert_eq!(col, "c");
                                assert_eq!(val, &WhereValue::Number(3.0));
                            }
                            _ => panic!("Wrong AND right"),
                        }
                    }
                    _ => panic!("Wrong right expression"),
                }
            }
            _ => panic!("Wrong top-level expression"),
        }

        // With parentheses: (a = 1 OR b = 2) AND c = 3
        let expr2 = WhereParser::parse("(a = 1 OR b = 2) AND c = 3").unwrap();
        match expr2 {
            WhereExpr::And(left, right) => {
                // Left should be (a = 1 OR b = 2)
                match left.as_ref() {
                    WhereExpr::Or(l, r) => {
                        match l.as_ref() {
                            WhereExpr::Equal(col, val) => {
                                assert_eq!(col, "a");
                                assert_eq!(val, &WhereValue::Number(1.0));
                            }
                            _ => panic!("Wrong OR left"),
                        }
                        match r.as_ref() {
                            WhereExpr::Equal(col, val) => {
                                assert_eq!(col, "b");
                                assert_eq!(val, &WhereValue::Number(2.0));
                            }
                            _ => panic!("Wrong OR right"),
                        }
                    }
                    _ => panic!("Wrong left expression"),
                }
                // Right should be c = 3
                match right.as_ref() {
                    WhereExpr::Equal(col, val) => {
                        assert_eq!(col, "c");
                        assert_eq!(val, &WhereValue::Number(3.0));
                    }
                    _ => panic!("Wrong right expression"),
                }
            }
            _ => panic!("Wrong top-level expression"),
        }
    }

    #[test]
    fn test_case_conversion_methods() {
        // Test ToLower
        let expr = WhereParser::parse("executionSide.ToLower() = \"buy\"").unwrap();
        match expr {
            WhereExpr::ToLower(col, op, val) => {
                assert_eq!(col, "executionSide");
                assert_eq!(op, ComparisonOp::Equal);
                assert_eq!(val, "buy");
            }
            _ => panic!("Wrong expression type for ToLower"),
        }

        // Test ToUpper
        let expr = WhereParser::parse("status.ToUpper() != \"PENDING\"").unwrap();
        match expr {
            WhereExpr::ToUpper(col, op, val) => {
                assert_eq!(col, "status");
                assert_eq!(op, ComparisonOp::NotEqual);
                assert_eq!(val, "PENDING");
            }
            _ => panic!("Wrong expression type for ToUpper"),
        }
    }
}
