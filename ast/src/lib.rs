#![allow(dead_code)]
use parser::{AtomKind, ParseError, SExpr, StringReader, Token, TokenKind};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::{io, path::Path};

#[derive(Debug)]
pub enum TranspileError {
    ParseError(ParseError),
    IoError(io::Error),
}

impl From<ParseError> for TranspileError {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl From<io::Error> for TranspileError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

pub struct Pythonify<'a> {
    src: &'a str,
    parser: StringReader<'a>,
}

impl<'a> Pythonify<'a> {
    pub fn new(src: &'a str) -> Pythonify<'a> {
        Self {
            src,
            parser: StringReader::new(src),
        }
    }

    pub fn output(mut self, path: impl AsRef<Path>) -> Result<(), TranspileError> {
        let mut file = BufWriter::new(File::create(path)?);

        let token = self.parser.next_token()?;
        let string = self.transpile(token, String::new())?;
        println!("{string}");
        file.write_all(string.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    fn transpile(&mut self, token: Token, string: String) -> Result<String, TranspileError> {
        self.parser.dbg_token(&token);
        match token.kind {
            TokenKind::SExpr(ref _s) => {
                let sexpr = self.transpile_sexpr(&token, string)?;
                let token = self.parser.next_token()?;
                self.transpile(token, sexpr)
            }
            TokenKind::EOF => Ok(string),
            _ => todo!(),
        }
    }

    fn transpile_sexpr(&mut self, token: &Token, string: String) -> Result<String, TranspileError> {
        let sexpr = match token.kind {
            TokenKind::SExpr(ref s) => s,
            _ => return self.transpose_non_cons(token, string),
        };
        match sexpr {
            SExpr::Cons { car, cdr } => match car.kind {
                TokenKind::SExpr(_) => {
                    let func = self.transpile_sexpr(car, string)?;
                    let args = {
                        let mut args_vec = vec![];
                        for token in cdr {
                            args_vec.push(self.transpile_sexpr(token, String::new())?)
                        }
                        args_vec.join(" ").trim().to_owned()
                    };
                    Ok(format!("{func}({args})"))
                }
                TokenKind::Atom(AtomKind::Literal(ref literal)) => {
                    Ok(format!("{string}\n{}", literal.as_str(self.src)))
                }
                TokenKind::Atom(AtomKind::Symbol(ref symbol, _)) => {
                    let args = {
                        let mut args_vec = vec![];
                        for token in cdr {
                            //self.parser.dbg_token(token);
                            args_vec.push(self.transpile_sexpr(token, String::new())?)
                        }
                        args_vec.join(" ").trim().to_owned()
                    };
                    Ok(format!(
                        "{string}\n{func}({args})",
                        func = symbol.as_str(self.src),
                    ))
                }
                TokenKind::Nil => Ok(string),
                TokenKind::ListNil => Ok(format!("{string}()")),
                TokenKind::EOF => Ok(string),
            },
            SExpr::Nil => Ok(string),
        }
    }

    fn transpose_non_cons(
        &mut self,
        token: &Token,
        string: String,
    ) -> Result<String, TranspileError> {
        match token.kind {
            TokenKind::Atom(AtomKind::Literal(ref literal)) => {
                Ok(literal.as_str(self.src).to_string())
            }
            TokenKind::Atom(AtomKind::Symbol(ref symbol, _)) => {
                Ok(symbol.as_str(self.src).to_string())
            }
            TokenKind::Nil => Ok(string),
            TokenKind::ListNil => Ok("()".to_string()),
            TokenKind::EOF => Ok(string),
            _ => unreachable!(),
        }
    }
}
