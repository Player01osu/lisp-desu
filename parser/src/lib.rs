#![allow(dead_code)]
use lexer::{Cursor, Span, Token as LexerToken, TokenKind as LexerTokenKind};

pub struct Parser<'a> {
    pub string_reader: StringReader<'a>,
}

pub struct StringReader<'a> {
    pub src: &'a str,
    pub cursor: Cursor<'a>,
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    SExpr(SExpr),
    Atom(AtomKind),
    Nil,
    ListNil,
    EOF,
}

#[derive(Debug, Clone)]
pub enum SExpr {
    Cons { car: Box<Token>, cdr: Vec<Token> },
    Nil,
}

#[derive(Debug, Clone)]
pub enum AtomKind {
    Literal(LexerToken),
    Symbol(LexerToken),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Vec<LexerTokenKind>, LexerToken),
}

impl ParseError {
    fn expected(expect: &[LexerTokenKind], got: LexerToken) -> Self {
        Self::UnexpectedToken(expect.to_vec(), got)
    }
}

const CONS_KINDS: [LexerTokenKind; 13] = [
    LexerTokenKind::Keyword,
    LexerTokenKind::Ident,
    LexerTokenKind::Backquote,
    LexerTokenKind::Literal,
    LexerTokenKind::OpenParen,
    LexerTokenKind::CloseParen,
    LexerTokenKind::OpenParen,
    LexerTokenKind::CloseParen,
    LexerTokenKind::OpenAngleBracket,
    LexerTokenKind::CloseAngleBracket,
    LexerTokenKind::Eq,
    LexerTokenKind::Bang,
    LexerTokenKind::And,
];

impl<'a> StringReader<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            cursor: Cursor::new(src),
        }
    }

    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        let lexer_token = self.cursor.next_token();
        match lexer_token.kind() {
            LexerTokenKind::Whitespace | LexerTokenKind::LineComment => self.next_token(),
            LexerTokenKind::OpenParen => self.parse_sexpr(lexer_token),
            LexerTokenKind::CloseParen => todo!(),
            LexerTokenKind::Keyword => todo!(),
            LexerTokenKind::Ident => todo!(),
            LexerTokenKind::OpenAngleBracket => todo!(),
            LexerTokenKind::CloseAngleBracket => todo!(),
            LexerTokenKind::Eq => todo!(),
            LexerTokenKind::Bang => todo!(),
            LexerTokenKind::And => todo!(),
            LexerTokenKind::Backquote => todo!(),
            LexerTokenKind::Literal => todo!(),
            LexerTokenKind::Dummy => todo!(),
            LexerTokenKind::Comma => todo!(),
            LexerTokenKind::EOF => Ok(Token {
                kind: TokenKind::EOF,
                span: Span::default(),
            }),
        }
    }

    fn parse_sexpr(&mut self, lexer_token: LexerToken) -> Result<Token, ParseError> {
        let start_row = lexer_token.span.start_row;
        let start_col = lexer_token.span.start_col;

        let car_lexer = self.expect_next(&CONS_KINDS)?;
        let cell = self.parse_cell(car_lexer)?;
        let car = match cell.kind {
            TokenKind::Nil => {
                return Ok(Token {
                    kind: TokenKind::ListNil,
                    span: Span::new(
                        lexer_token.span.start_row,
                        cell.span.end_row,
                        lexer_token.span.start_col,
                        cell.span.end_col,
                    ),
                })
            }
            _ => Box::new(cell),
        };

        let mut cdr_tokens = vec![];
        loop {
            let cdr_lexer = self.expect_next(&CONS_KINDS)?;
            let cdr = self.parse_cell(cdr_lexer)?;
            let is_nil = matches!(cdr.kind, TokenKind::Nil);
            cdr_tokens.push(cdr);
            if is_nil {
                break;
            }
        }

        let end_span = cdr_tokens
            .iter()
            .last()
            .expect("At least one element should exist.")
            .span;

        let (end_row, end_col) = (end_span.end_row, end_span.end_col);
        let span = Span::new(start_row, end_row, start_col, end_col);
        Ok(Token {
            kind: TokenKind::SExpr(SExpr::Cons {
                car,
                cdr: cdr_tokens,
            }),
            span,
        })
    }

    fn parse_cell(&mut self, lexer_token: LexerToken) -> Result<Token, ParseError> {
        Ok(match lexer_token.kind {
            // Recurse into S-Expr production
            LexerTokenKind::OpenParen => self.parse_sexpr(lexer_token)?,
            LexerTokenKind::CloseParen => Token {
                span: lexer_token.span,
                kind: TokenKind::Nil,
            },
            // Atom
            LexerTokenKind::Ident | LexerTokenKind::Keyword => Token {
                span: lexer_token.span,
                kind: TokenKind::Atom(AtomKind::Symbol(lexer_token)),
            },
            _ => Token {
                span: lexer_token.span,
                kind: TokenKind::Atom(AtomKind::Literal(lexer_token)),
            },
        })
    }

    fn next_lexer(&mut self) -> LexerToken {
        loop {
            let token = self.cursor.next_token();
            if token.kind != LexerTokenKind::Whitespace {
                break token;
            }
        }
    }

    fn expect_next(&mut self, expected_kind: &[LexerTokenKind]) -> Result<LexerToken, ParseError> {
        let lexer_token = self.next_lexer();
        for expected in expected_kind {
            match lexer_token.kind {
                kind if kind == *expected => return Ok(lexer_token),
                _ => continue,
            }
        }
        Err(ParseError::expected(expected_kind, lexer_token))
    }

    // TODO
    pub fn dbg_token(&self, token: &Token) {
        // {STRING} {KIND} {SPAN}
        dbg!(token.span);
        self.print_token(token);
    }
    fn print_token(&self, token: &Token) {
        match token.kind {
            TokenKind::SExpr(ref s) => match s {
                SExpr::Cons { car, cdr } => {
                    self.print_token(car);
                    for t in cdr {
                        self.print_token(t);
                    }
                }
                SExpr::Nil => {
                    dbg!(s);
                }
            },
            TokenKind::Atom(ref a) => match a {
                AtomKind::Literal(literal) => {
                    self.cursor.dbg_token(literal);
                }
                AtomKind::Symbol(symbol) => {
                    self.cursor.dbg_token(symbol);
                }
            },
            TokenKind::Nil => {
                dbg!(token);
            }
            TokenKind::ListNil => {
                dbg!(token);
            }
            TokenKind::EOF => {
                dbg!(token);
            }
        }
    }
}
