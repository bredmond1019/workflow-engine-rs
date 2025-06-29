//! # Template Parser
//!
//! This module provides parsing functionality for template syntax,
//! converting template strings into an abstract syntax tree (AST).

use super::{TemplateError, VariableType};
use std::collections::HashMap;

/// Template Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateAst {
    /// Plain text content
    Text(String),
    /// Variable interpolation {{variable}}
    Variable {
        name: String,
        filters: Vec<String>,
    },
    /// Conditional block {{#if condition}}...{{/if}}
    Conditional {
        condition: Box<TemplateAst>,
        then_branch: Box<TemplateAst>,
        else_branch: Option<Box<TemplateAst>>,
    },
    /// Loop block {{#each items}}...{{/each}}
    Loop {
        variable: String,
        body: Box<TemplateAst>,
    },
    /// Include another template {{> template_name}}
    Include {
        template: String,
        context: HashMap<String, String>,
    },
    /// Block of statements
    Block(Vec<TemplateAst>),
    /// Helper function call {{helper arg1 arg2}}
    Helper {
        name: String,
        args: Vec<String>,
    },
}

/// Parse error types
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Invalid syntax at position {position}: {message}")]
    InvalidSyntax { position: usize, message: String },
    
    #[error("Unclosed block: {block_type}")]
    UnclosedBlock { block_type: String },
    
    #[error("Unknown helper function: {name}")]
    UnknownHelper { name: String },
    
    #[error("Invalid variable name: {name}")]
    InvalidVariable { name: String },
}

impl From<ParseError> for TemplateError {
    fn from(error: ParseError) -> Self {
        TemplateError::ParseError {
            message: error.to_string(),
        }
    }
}

/// Template parser
pub struct TemplateParser {
    /// Registered helper functions
    helpers: HashMap<String, HelperDefinition>,
    /// Allowed variable name pattern
    variable_pattern: regex::Regex,
}

/// Helper function definition
#[derive(Debug, Clone)]
struct HelperDefinition {
    name: String,
    min_args: usize,
    max_args: Option<usize>,
}

impl TemplateParser {
    /// Create a new parser with default helpers
    pub fn new() -> Self {
        let mut helpers = HashMap::new();
        
        // Register built-in helpers
        helpers.insert("if".to_string(), HelperDefinition {
            name: "if".to_string(),
            min_args: 1,
            max_args: Some(1),
        });
        
        helpers.insert("each".to_string(), HelperDefinition {
            name: "each".to_string(),
            min_args: 1,
            max_args: Some(1),
        });
        
        helpers.insert("json".to_string(), HelperDefinition {
            name: "json".to_string(),
            min_args: 1,
            max_args: Some(1),
        });
        
        helpers.insert("uppercase".to_string(), HelperDefinition {
            name: "uppercase".to_string(),
            min_args: 1,
            max_args: Some(1),
        });
        
        helpers.insert("lowercase".to_string(), HelperDefinition {
            name: "lowercase".to_string(),
            min_args: 1,
            max_args: Some(1),
        });
        
        Self {
            helpers,
            variable_pattern: regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$")
                .expect("Failed to compile variable pattern regex - this is a bug"),
        }
    }
    
    /// Parse a template string into AST
    pub fn parse(&self, template: &str) -> Result<TemplateAst, ParseError> {
        let tokens = self.tokenize(template)?;
        self.parse_tokens(&tokens)
    }
    
    /// Extract variables from template
    pub fn extract_variables(&self, template: &str) -> Result<HashMap<String, VariableType>, ParseError> {
        let ast = self.parse(template)?;
        let mut variables = HashMap::new();
        self.extract_variables_from_ast(&ast, &mut variables);
        Ok(variables)
    }
    
    /// Tokenize template string
    fn tokenize(&self, template: &str) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        let mut chars = template.chars().peekable();
        let mut position = 0;
        let mut text_buffer = String::new();
        
        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                // Save any accumulated text
                if !text_buffer.is_empty() {
                    tokens.push(Token::Text(text_buffer.clone()));
                    text_buffer.clear();
                }
                
                // Consume second '{'
                chars.next();
                position += 2;
                
                // Parse template expression
                let expr = self.parse_expression(&mut chars, &mut position)?;
                tokens.push(expr);
            } else {
                text_buffer.push(ch);
                position += 1;
            }
        }
        
        // Save any remaining text
        if !text_buffer.is_empty() {
            tokens.push(Token::Text(text_buffer));
        }
        
        Ok(tokens)
    }
    
    /// Parse expression within {{ }}
    fn parse_expression(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        position: &mut usize,
    ) -> Result<Token, ParseError> {
        let mut expr = String::new();
        let start_pos = *position;
        
        // Read until }}
        loop {
            match chars.next() {
                Some('}') if chars.peek() == Some(&'}') => {
                    chars.next(); // Consume second }
                    *position += 2;
                    break;
                }
                Some(ch) => {
                    expr.push(ch);
                    *position += 1;
                }
                None => {
                    return Err(ParseError::InvalidSyntax {
                        position: start_pos,
                        message: "Unclosed template expression".to_string(),
                    });
                }
            }
        }
        
        // Parse expression type
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            return Err(ParseError::InvalidSyntax {
                position: start_pos,
                message: "Empty template expression".to_string(),
            });
        }
        
        // Check for special forms
        if trimmed.starts_with('#') {
            // Block start
            let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();
            if parts.is_empty() {
                return Err(ParseError::InvalidSyntax {
                    position: start_pos,
                    message: "Invalid block syntax".to_string(),
                });
            }
            
            match parts[0] {
                "if" => Ok(Token::IfStart(parts.get(1).unwrap_or(&"").to_string())),
                "each" => Ok(Token::EachStart(parts.get(1).unwrap_or(&"").to_string())),
                _ => Err(ParseError::UnknownHelper {
                    name: parts[0].to_string(),
                }),
            }
        } else if trimmed.starts_with('/') {
            // Block end
            let block_type = trimmed[1..].trim();
            match block_type {
                "if" => Ok(Token::IfEnd),
                "each" => Ok(Token::EachEnd),
                _ => Err(ParseError::InvalidSyntax {
                    position: start_pos,
                    message: format!("Unknown block end: {}", block_type),
                }),
            }
        } else if trimmed == "else" {
            Ok(Token::Else)
        } else if trimmed.starts_with('>') {
            // Include
            let template_name = trimmed[1..].trim();
            Ok(Token::Include(template_name.to_string()))
        } else {
            // Variable or helper
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() == 1 && self.variable_pattern.is_match(parts[0]) {
                Ok(Token::Variable(parts[0].to_string()))
            } else if parts.len() > 1 {
                Ok(Token::Helper {
                    name: parts[0].to_string(),
                    args: parts[1..].iter().map(|s| s.to_string()).collect(),
                })
            } else {
                Err(ParseError::InvalidVariable {
                    name: trimmed.to_string(),
                })
            }
        }
    }
    
    /// Parse tokens into AST
    fn parse_tokens(&self, tokens: &[Token]) -> Result<TemplateAst, ParseError> {
        let mut parser = TokenParser::new(tokens);
        parser.parse()
    }
    
    /// Extract variables from AST
    fn extract_variables_from_ast(&self, ast: &TemplateAst, variables: &mut HashMap<String, VariableType>) {
        match ast {
            TemplateAst::Variable { name, .. } => {
                variables.insert(name.clone(), VariableType::Any);
            }
            TemplateAst::Conditional { condition, then_branch, else_branch } => {
                self.extract_variables_from_ast(condition, variables);
                self.extract_variables_from_ast(then_branch, variables);
                if let Some(else_ast) = else_branch {
                    self.extract_variables_from_ast(else_ast, variables);
                }
            }
            TemplateAst::Loop { variable, body } => {
                variables.insert(variable.clone(), VariableType::Array(Box::new(VariableType::Any)));
                self.extract_variables_from_ast(body, variables);
            }
            TemplateAst::Block(statements) => {
                for stmt in statements {
                    self.extract_variables_from_ast(stmt, variables);
                }
            }
            TemplateAst::Helper { args, .. } => {
                for arg in args {
                    if self.variable_pattern.is_match(arg) {
                        variables.insert(arg.clone(), VariableType::Any);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Token types for parsing
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Text(String),
    Variable(String),
    IfStart(String),
    IfEnd,
    Else,
    EachStart(String),
    EachEnd,
    Include(String),
    Helper { name: String, args: Vec<String> },
}

/// Token parser for building AST
struct TokenParser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> TokenParser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, position: 0 }
    }
    
    /// Helper to convert a vector of AST nodes into a single AST node
    fn statements_to_ast(statements: Vec<TemplateAst>) -> TemplateAst {
        match statements.len() {
            0 => TemplateAst::Text(String::new()),
            1 => statements.into_iter().next()
                .expect("statements.len() == 1 but no element found - this is a bug"),
            _ => TemplateAst::Block(statements),
        }
    }
    
    /// Helper to convert a vector of AST nodes into a boxed AST node
    fn statements_to_boxed_ast(statements: Vec<TemplateAst>) -> Box<TemplateAst> {
        Box::new(Self::statements_to_ast(statements))
    }
    
    fn parse(&mut self) -> Result<TemplateAst, ParseError> {
        let mut statements = Vec::new();
        
        while self.position < self.tokens.len() {
            statements.push(self.parse_statement()?);
        }
        
        Ok(Self::statements_to_ast(statements))
    }
    
    fn parse_statement(&mut self) -> Result<TemplateAst, ParseError> {
        match &self.tokens.get(self.position) {
            Some(Token::Text(text)) => {
                self.position += 1;
                Ok(TemplateAst::Text(text.clone()))
            }
            Some(Token::Variable(name)) => {
                self.position += 1;
                Ok(TemplateAst::Variable {
                    name: name.clone(),
                    filters: Vec::new(),
                })
            }
            Some(Token::IfStart(condition)) => {
                self.parse_if_block(condition.clone())
            }
            Some(Token::EachStart(variable)) => {
                self.parse_each_block(variable.clone())
            }
            Some(Token::Include(template)) => {
                self.position += 1;
                Ok(TemplateAst::Include {
                    template: template.clone(),
                    context: HashMap::new(),
                })
            }
            Some(Token::Helper { name, args }) => {
                self.position += 1;
                Ok(TemplateAst::Helper {
                    name: name.clone(),
                    args: args.clone(),
                })
            }
            Some(token) => {
                Err(ParseError::InvalidSyntax {
                    position: self.position,
                    message: format!("Unexpected token: {:?}", token),
                })
            }
            None => Err(ParseError::UnexpectedEof),
        }
    }
    
    fn parse_if_block(&mut self, condition: String) -> Result<TemplateAst, ParseError> {
        self.position += 1; // Skip IfStart
        
        let mut then_statements = Vec::new();
        let mut else_statements = None;
        let mut in_else = false;
        
        while self.position < self.tokens.len() {
            match &self.tokens[self.position] {
                Token::IfEnd => {
                    self.position += 1;
                    
                    let then_branch = Self::statements_to_boxed_ast(then_statements);
                    
                    let else_branch = else_statements.map(Self::statements_to_boxed_ast);
                    
                    return Ok(TemplateAst::Conditional {
                        condition: Box::new(TemplateAst::Variable {
                            name: condition,
                            filters: Vec::new(),
                        }),
                        then_branch,
                        else_branch,
                    });
                }
                Token::Else => {
                    self.position += 1;
                    in_else = true;
                    else_statements = Some(Vec::new());
                }
                _ => {
                    let stmt = self.parse_statement()?;
                    if in_else {
                        if let Some(ref mut else_stmts) = else_statements {
                            else_stmts.push(stmt);
                        } else {
                            return Err(ParseError::InvalidSyntax {
                                position: self.position,
                                message: "Else branch without else token".to_string(),
                            });
                        }
                    } else {
                        then_statements.push(stmt);
                    }
                }
            }
        }
        
        Err(ParseError::UnclosedBlock {
            block_type: "if".to_string(),
        })
    }
    
    fn parse_each_block(&mut self, variable: String) -> Result<TemplateAst, ParseError> {
        self.position += 1; // Skip EachStart
        
        let mut body_statements = Vec::new();
        
        while self.position < self.tokens.len() {
            match &self.tokens[self.position] {
                Token::EachEnd => {
                    self.position += 1;
                    
                    let body = Self::statements_to_boxed_ast(body_statements);
                    
                    return Ok(TemplateAst::Loop { variable, body });
                }
                _ => {
                    body_statements.push(self.parse_statement()?);
                }
            }
        }
        
        Err(ParseError::UnclosedBlock {
            block_type: "each".to_string(),
        })
    }
}

impl Default for TemplateParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_text() {
        let parser = TemplateParser::new();
        let ast = parser.parse("Hello World").unwrap();
        assert_eq!(ast, TemplateAst::Text("Hello World".to_string()));
    }
    
    #[test]
    fn test_parse_variable() {
        let parser = TemplateParser::new();
        let ast = parser.parse("Hello {{name}}!").unwrap();
        
        match ast {
            TemplateAst::Block(stmts) => {
                assert_eq!(stmts.len(), 3);
                assert_eq!(stmts[0], TemplateAst::Text("Hello ".to_string()));
                assert_eq!(stmts[1], TemplateAst::Variable {
                    name: "name".to_string(),
                    filters: Vec::new(),
                });
                assert_eq!(stmts[2], TemplateAst::Text("!".to_string()));
            }
            _ => panic!("Expected Block"),
        }
    }
    
    #[test]
    fn test_parse_conditional() {
        let parser = TemplateParser::new();
        let template = "{{#if active}}Active{{else}}Inactive{{/if}}";
        let ast = parser.parse(template).unwrap();
        
        match ast {
            TemplateAst::Conditional { condition, then_branch, else_branch } => {
                match condition.as_ref() {
                    TemplateAst::Variable { name, .. } => assert_eq!(name, "active"),
                    _ => panic!("Expected Variable"),
                }
                assert!(matches!(then_branch.as_ref(), TemplateAst::Text(s) if s == "Active"));
                assert!(matches!(else_branch.as_ref().unwrap().as_ref(), TemplateAst::Text(s) if s == "Inactive"));
            }
            _ => panic!("Expected Conditional"),
        }
    }
    
    #[test]
    fn test_parse_loop() {
        let parser = TemplateParser::new();
        let template = "{{#each items}}{{this}}{{/each}}";
        let ast = parser.parse(template).unwrap();
        
        match ast {
            TemplateAst::Loop { variable, body } => {
                assert_eq!(variable, "items");
                assert!(matches!(body.as_ref(), TemplateAst::Variable { name, .. } if name == "this"));
            }
            _ => panic!("Expected Loop"),
        }
    }
    
    #[test]
    fn test_extract_variables() {
        let parser = TemplateParser::new();
        let template = "Hello {{name}}, you have {{count}} items";
        let vars = parser.extract_variables(template).unwrap();
        
        assert_eq!(vars.len(), 2);
        assert!(vars.contains_key("name"));
        assert!(vars.contains_key("count"));
    }
}