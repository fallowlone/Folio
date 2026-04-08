pub mod ast;
pub mod resolver;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use ast::{Block, Content, Document, Value};
use crate::lexer::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    fn expect_ident(&mut self) -> String {
        match self.advance().clone() {
            Token::Ident(s) => s,
            t => panic!("Expected Ident, got {:?}", t),
        }
    }

    fn expect(&mut self, expected: &Token) {
        let t = self.advance().clone();
        if &t != expected {
            panic!("Expected {:?}, got {:?}", expected, t);
        }
    }

    pub fn parse(&mut self) -> Document {
        let mut vars = HashMap::new();
        let mut blocks = Vec::new();

        while self.current() != &Token::Eof {
            match self.current().clone() {
                Token::Ident(ref name) if name == "STYLES" => {
                    let style_vars = self.parse_styles();
                    vars.extend(style_vars);
                }
                Token::Ident(_) => {
                    blocks.push(self.parse_block());
                }
                _ => { self.advance(); }
            }
        }

        Document { vars, blocks }
    }

    // Parse STYLES({ #key: value, ... }) → HashMap
    fn parse_styles(&mut self) -> HashMap<String, Value> {
        self.expect_ident(); // consume "STYLES"
        self.expect(&Token::LParen);
        self.expect(&Token::LBrace);

        let mut vars = HashMap::new();

        while self.current() != &Token::RBrace && self.current() != &Token::Eof {
            if let Token::Hash(key) = self.current().clone() {
                self.advance();
                self.expect(&Token::Colon);
                let value = self.parse_value();
                vars.insert(key, value);
            } else if self.current() == &Token::Comma {
                self.advance();
            } else {
                self.advance();
            }
        }

        self.expect(&Token::RBrace);
        self.expect(&Token::RParen);

        vars
    }

    // Parse a value token into a Value
    fn parse_value(&mut self) -> Value {
        match self.advance().clone() {
            Token::String(s) => Value::Str(s),
            Token::Number(n) => Value::Number(n),
            Token::Unit(n, u) => Value::Unit(n, u),
            Token::Hash(s) => {
                // #FF0000 is a color, #name is a variable
                if s.chars().all(|c| c.is_ascii_hexdigit()) && s.len() == 6 {
                    Value::Color(s)
                } else {
                    Value::Var(s)
                }
            }
            t => panic!("Expected value, got {:?}", t),
        }
    }

    // Parse attrs: { key: value, key: value }
    fn parse_attrs(&mut self) -> HashMap<String, Value> {
        self.expect(&Token::LBrace);
        let mut attrs = HashMap::new();

        while self.current() != &Token::RBrace && self.current() != &Token::Eof {
            match self.current().clone() {
                Token::Ident(key) => {
                    self.advance();
                    self.expect(&Token::Colon);
                    let value = self.parse_value();
                    attrs.insert(key, value);
                }
                Token::Comma => { self.advance(); }
                _ => { self.advance(); }
            }
        }

        self.expect(&Token::RBrace);
        attrs
    }

    // Parse a block: IDENT({attrs} content) or IDENT(content)
    fn parse_block(&mut self) -> Block {
        let kind = self.expect_ident();
        self.expect(&Token::LParen);

        let attrs = if self.current() == &Token::LBrace {
            self.parse_attrs()
        } else {
            HashMap::new()
        };

        let content = self.parse_content();

        self.expect(&Token::RParen);

        Block { kind, attrs, content }
    }

    // Parse content: text or nested blocks until RParen
    fn parse_content(&mut self) -> Content {
        match self.current().clone() {
            Token::RParen => Content::Empty,
            Token::Text(s) => {
                self.advance();
                Content::Text(s)
            }
            Token::Ident(_) => {
                // nested blocks
                let mut blocks = Vec::new();
                while self.current() != &Token::RParen && self.current() != &Token::Eof {
                    match self.current().clone() {
                        Token::Ident(ref name) if name == "STYLES" => {
                            // page-level STYLES — skip for now
                            self.parse_styles();
                        }
                        Token::Ident(_) => {
                            blocks.push(self.parse_block());
                        }
                        _ => { self.advance(); }
                    }
                }
                Content::Blocks(blocks)
            }
            _ => {
                self.advance();
                Content::Empty
            }
        }
    }
}
