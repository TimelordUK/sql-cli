/// FILE CTE Parser Module
///
/// Parses FILE CTE specifications for enumerating filesystem metadata as a
/// virtual table. See `docs/FILE_CTE_DESIGN.md` for design rationale.
///
/// Syntax:
/// ```sql
/// WITH files AS (
///     FILE PATH './data'
///     [RECURSIVE]
///     [GLOB '*.csv']
///     [MAX_DEPTH 5]
///     [MAX_FILES 100000]
///     [FOLLOW_LINKS]
///     [INCLUDE_HIDDEN]
/// )
/// SELECT ... FROM files;
/// ```
use super::ast::FileCTESpec;
use super::lexer::Token;

pub struct FileCteParser;

impl FileCteParser {
    /// Main entry point to parse a FILE CTE specification.
    ///
    /// Called after the opening `(` and the `FILE` keyword have been consumed.
    /// Parses clauses until hitting the closing `)`.
    pub fn parse(parser: &mut crate::sql::recursive_parser::Parser) -> Result<FileCTESpec, String> {
        // Expect PATH keyword
        if let Token::Identifier(id) = &parser.current_token {
            if id.to_uppercase() != "PATH" {
                return Err(format!("Expected PATH keyword in FILE CTE, found '{}'", id));
            }
        } else {
            return Err("Expected PATH keyword in FILE CTE".to_string());
        }
        parser.advance();

        // Parse PATH string
        let path = match &parser.current_token {
            Token::StringLiteral(p) => p.clone(),
            _ => return Err("Expected string literal after PATH keyword".to_string()),
        };
        parser.advance();

        // Initialize optional fields with defaults
        let mut recursive = false;
        let mut glob: Option<String> = None;
        let mut max_depth: Option<usize> = None;
        let mut max_files: Option<usize> = None;
        let mut follow_links = false;
        let mut include_hidden = false;

        // Parse optional clauses until closing parenthesis
        while !matches!(parser.current_token, Token::RightParen)
            && !matches!(parser.current_token, Token::Eof)
        {
            if let Token::Identifier(id) = &parser.current_token {
                match id.to_uppercase().as_str() {
                    "RECURSIVE" => {
                        parser.advance();
                        recursive = true;
                    }
                    "GLOB" => {
                        parser.advance();
                        glob = Some(Self::parse_string(parser, "GLOB")?);
                    }
                    "MAX_DEPTH" => {
                        parser.advance();
                        max_depth = Some(Self::parse_usize(parser, "MAX_DEPTH")?);
                    }
                    "MAX_FILES" => {
                        parser.advance();
                        max_files = Some(Self::parse_usize(parser, "MAX_FILES")?);
                    }
                    "FOLLOW_LINKS" => {
                        parser.advance();
                        follow_links = true;
                    }
                    "INCLUDE_HIDDEN" => {
                        parser.advance();
                        include_hidden = true;
                    }
                    _ => {
                        return Err(format!(
                            "Unexpected keyword '{}' in FILE CTE specification",
                            id
                        ));
                    }
                }
            } else {
                break;
            }
        }

        Ok(FileCTESpec {
            path,
            recursive,
            glob,
            max_depth,
            max_files,
            follow_links,
            include_hidden,
        })
    }

    fn parse_string(
        parser: &mut crate::sql::recursive_parser::Parser,
        clause: &str,
    ) -> Result<String, String> {
        match &parser.current_token {
            Token::StringLiteral(s) => {
                let s = s.clone();
                parser.advance();
                Ok(s)
            }
            _ => Err(format!("Expected string literal after {} clause", clause)),
        }
    }

    fn parse_usize(
        parser: &mut crate::sql::recursive_parser::Parser,
        clause: &str,
    ) -> Result<usize, String> {
        match &parser.current_token {
            Token::NumberLiteral(n) => {
                let val = n
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid {} value: {}", clause, n))?;
                parser.advance();
                Ok(val)
            }
            _ => Err(format!("Expected integer after {} clause", clause)),
        }
    }
}
