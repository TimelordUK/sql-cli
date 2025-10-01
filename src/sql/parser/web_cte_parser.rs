/// Web CTE Parser Module
/// Handles parsing of WEB CTEs for HTTP data fetching with custom selectors
use super::ast::{DataFormat, HttpMethod, WebCTESpec};
use super::lexer::Token;

pub struct WebCteParser<'a> {
    _tokens: &'a mut dyn Iterator<Item = Token>,
    _current_token: Token,
}

impl<'a> WebCteParser<'a> {
    pub fn new(tokens: &'a mut dyn Iterator<Item = Token>, current_token: Token) -> Self {
        Self {
            _tokens: tokens,
            _current_token: current_token,
        }
    }

    /// Main entry point to parse WEB CTE specification
    /// Expects: URL 'url' [METHOD method] [FORMAT format] [HEADERS (...)] [CACHE n] [BODY 'body'] [FORM_FILE 'field' 'path'] [FORM_FIELD 'field' 'value']
    pub fn parse(parser: &mut crate::sql::recursive_parser::Parser) -> Result<WebCTESpec, String> {
        // Expect URL keyword
        if let Token::Identifier(id) = &parser.current_token {
            if id.to_uppercase() != "URL" {
                return Err("Expected URL keyword in WEB CTE".to_string());
            }
        } else {
            return Err("Expected URL keyword in WEB CTE".to_string());
        }
        parser.advance();

        // Parse URL string
        let url = match &parser.current_token {
            Token::StringLiteral(url) => url.clone(),
            _ => return Err("Expected URL string after URL keyword".to_string()),
        };
        parser.advance();

        // Initialize optional fields
        let mut format = None;
        let mut headers = Vec::new();
        let mut cache_seconds = None;
        let mut method = None;
        let mut body = None;
        let mut json_path = None;
        let mut form_files = Vec::new();
        let mut form_fields = Vec::new();

        // Parse optional clauses until we hit the closing parenthesis
        while !matches!(parser.current_token, Token::RightParen)
            && !matches!(parser.current_token, Token::Eof)
        {
            if let Token::Identifier(id) = &parser.current_token {
                match id.to_uppercase().as_str() {
                    "FORMAT" => {
                        parser.advance();
                        format = Some(Self::parse_data_format(parser)?);
                    }
                    "CACHE" => {
                        parser.advance();
                        cache_seconds = Some(Self::parse_cache_duration(parser)?);
                    }
                    "HEADERS" => {
                        parser.advance();
                        headers = Self::parse_headers(parser)?;
                    }
                    "METHOD" => {
                        parser.advance();
                        method = Some(Self::parse_http_method(parser)?);
                    }
                    "BODY" => {
                        parser.advance();
                        body = Some(Self::parse_body(parser)?);
                    }
                    "JSON_PATH" => {
                        parser.advance();
                        json_path = Some(Self::parse_json_path(parser)?);
                    }
                    "FORM_FILE" => {
                        parser.advance();
                        let (field_name, file_path) = Self::parse_form_file(parser)?;
                        form_files.push((field_name, file_path));
                    }
                    "FORM_FIELD" => {
                        parser.advance();
                        let (field_name, value) = Self::parse_form_field(parser)?;
                        form_fields.push((field_name, value));
                    }
                    _ => {
                        return Err(format!(
                            "Unexpected keyword '{}' in WEB CTE specification",
                            id
                        ));
                    }
                }
            } else {
                break;
            }
        }

        Ok(WebCTESpec {
            url,
            format,
            headers,
            cache_seconds,
            method,
            body,
            json_path,
            form_files,
            form_fields,
        })
    }

    fn parse_data_format(
        parser: &mut crate::sql::recursive_parser::Parser,
    ) -> Result<DataFormat, String> {
        if let Token::Identifier(id) = &parser.current_token {
            let format = match id.to_uppercase().as_str() {
                "CSV" => DataFormat::CSV,
                "JSON" => DataFormat::JSON,
                "AUTO" => DataFormat::Auto,
                _ => return Err(format!("Unknown data format: {}", id)),
            };
            parser.advance();
            Ok(format)
        } else {
            Err("Expected data format (CSV, JSON, or AUTO)".to_string())
        }
    }

    fn parse_cache_duration(
        parser: &mut crate::sql::recursive_parser::Parser,
    ) -> Result<u64, String> {
        match &parser.current_token {
            Token::NumberLiteral(n) => {
                let duration = n
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid cache duration: {}", n))?;
                parser.advance();
                Ok(duration)
            }
            _ => Err("Expected number for cache duration".to_string()),
        }
    }

    fn parse_http_method(
        parser: &mut crate::sql::recursive_parser::Parser,
    ) -> Result<HttpMethod, String> {
        if let Token::Identifier(id) = &parser.current_token {
            let method = match id.to_uppercase().as_str() {
                "GET" => HttpMethod::GET,
                "POST" => HttpMethod::POST,
                "PUT" => HttpMethod::PUT,
                "DELETE" => HttpMethod::DELETE,
                "PATCH" => HttpMethod::PATCH,
                _ => return Err(format!("Unknown HTTP method: {}", id)),
            };
            parser.advance();
            Ok(method)
        } else {
            Err("Expected HTTP method (GET, POST, PUT, DELETE, PATCH)".to_string())
        }
    }

    fn parse_body(parser: &mut crate::sql::recursive_parser::Parser) -> Result<String, String> {
        match &parser.current_token {
            Token::StringLiteral(body) | Token::JsonBlock(body) => {
                let body = body.clone();
                parser.advance();
                Ok(body)
            }
            _ => Err("Expected string literal or $JSON$ block for BODY clause".to_string()),
        }
    }

    fn parse_json_path(
        parser: &mut crate::sql::recursive_parser::Parser,
    ) -> Result<String, String> {
        match &parser.current_token {
            Token::StringLiteral(path) => {
                let path = path.clone();
                parser.advance();
                Ok(path)
            }
            _ => Err("Expected string literal for JSON_PATH clause".to_string()),
        }
    }

    fn parse_form_file(
        parser: &mut crate::sql::recursive_parser::Parser,
    ) -> Result<(String, String), String> {
        // Parse field name
        let field_name = match &parser.current_token {
            Token::StringLiteral(name) => name.clone(),
            _ => return Err("Expected field name string after FORM_FILE".to_string()),
        };
        parser.advance();

        // Parse file path
        let file_path = match &parser.current_token {
            Token::StringLiteral(path) => path.clone(),
            _ => return Err("Expected file path string after field name".to_string()),
        };
        parser.advance();

        Ok((field_name, file_path))
    }

    fn parse_form_field(
        parser: &mut crate::sql::recursive_parser::Parser,
    ) -> Result<(String, String), String> {
        // Parse field name
        let field_name = match &parser.current_token {
            Token::StringLiteral(name) | Token::JsonBlock(name) => name.clone(),
            _ => return Err("Expected field name string after FORM_FIELD".to_string()),
        };
        parser.advance();

        // Parse field value (can be regular string or JSON block)
        let value = match &parser.current_token {
            Token::StringLiteral(val) | Token::JsonBlock(val) => val.clone(),
            _ => {
                return Err(
                    "Expected field value string or $JSON$ block after field name".to_string(),
                )
            }
        };
        parser.advance();

        Ok((field_name, value))
    }

    fn parse_headers(
        parser: &mut crate::sql::recursive_parser::Parser,
    ) -> Result<Vec<(String, String)>, String> {
        parser.consume(Token::LeftParen)?;
        let mut headers = Vec::new();

        loop {
            // Parse header name
            let key = match &parser.current_token {
                Token::Identifier(id) => id.clone(),
                Token::StringLiteral(s) => s.clone(),
                _ => return Err("Expected header name".to_string()),
            };
            parser.advance();

            // Expect : (colon) for header key-value separator
            if !matches!(parser.current_token, Token::Colon) {
                // For backwards compatibility, also accept =
                if matches!(parser.current_token, Token::Equal) {
                    parser.advance();
                } else {
                    return Err("Expected ':' or '=' after header name".to_string());
                }
            } else {
                parser.advance(); // consume the colon
            }

            // Parse header value
            let value = match &parser.current_token {
                Token::StringLiteral(s) => s.clone(),
                _ => return Err("Expected header value as string".to_string()),
            };
            parser.advance();

            headers.push((key, value));

            // Check for comma (more headers) or closing paren (end)
            if matches!(parser.current_token, Token::Comma) {
                parser.advance();
            } else if matches!(parser.current_token, Token::RightParen) {
                parser.advance();
                break;
            } else {
                return Err("Expected ',' or ')' after header value".to_string());
            }
        }

        Ok(headers)
    }
}
