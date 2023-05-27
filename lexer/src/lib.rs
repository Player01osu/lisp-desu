#![allow(dead_code)]
use std::io::stdout;
use std::io::Write;
use std::{fmt::Display, str::Chars};

#[derive(Debug, Clone)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
    pub len: usize,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Span {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

#[derive(Debug)]
pub struct Cursor<'a> {
    src: &'a str,
    chars: Chars<'a>,
    len_remaining: usize,
    buf: String,
    row: usize,
    col: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Keyword,
    Ident,
    Whitespace,
    Comma,

    /// '('
    OpenParen,
    /// ')'
    CloseParen,

    /// '<'
    OpenAngleBracket,
    /// '>'
    CloseAngleBracket,

    /// '='
    Eq,

    /// '!'
    Bang,

    /// '&'
    And,

    /// '`' or '\''
    Backquote,
    Literal,

    /// ';'
    LineComment,
    Dummy,
    EOF,
}

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n')
}

fn is_end_ident(c: char) -> bool {
    matches!(c, '(' | ')') || is_whitespace(c)
}

fn is_string_literal(c: char) -> bool {
    matches!(c, '"')
}

// TODO Pull this into proc macro.
//matches!(c, 'd' | 'a' | 'o' | 'n' | 'c' | 'i' | 'l')
//matches!(
//    s,
//    "defun" | "and" | "or" | "not" | "cond" | "nil" | "if" | "case"
//)
pub const KEYWORDS: [&str; 8] = ["defun", "and", "or", "not", "cond", "nil", "if", "case"];

fn is_keyword_prefix(c: char) -> bool {
    KEYWORDS
        .iter()
        .any(|w| (*w).chars().next().unwrap() == c)
}

fn is_keyword(s: &str) -> bool {
    KEYWORDS.iter().any(|w| s.eq(*w))
}

impl Token {
    pub fn new(kind: TokenKind, len: usize, span: Span) -> Self {
        Self { kind, len, span }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }
}

impl Span {
    pub fn new(start_row: usize, end_row: usize, start_col: usize, end_col: usize) -> Self {
        Self {
            start_row,
            end_row,
            start_col,
            end_col,
        }
    }

    pub fn make_span(
        start_row: usize,
        start_col: usize,
        start_chars: Chars,
        len: usize,
    ) -> (Self, usize, usize) {
        let newlines = start_chars.clone().take(len).filter(|c| *c == '\n').count();
        let last_char = start_chars.clone().take(len).last().unwrap();

        let end_row = match newlines {
            0 | 1 => start_row + newlines,
            _ => start_row + newlines - 1,
        };

        let end_col = start_chars
            .take(len)
            .fold((start_col - 1, '\0'), |(acc, prev), c| match prev {
                '\n' => (1, c),
                _ => (acc + 1, c),
            })
            .0;

        let advance_rows = newlines;
        let col = match last_char {
            '\n' => 1,
            _ => end_col + 1,
        };

        (
            Span::new(start_row, end_row, start_col, end_col),
            advance_rows,
            col,
        )
    }
}

impl<'a> Cursor<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            chars: src.chars(),
            len_remaining: src.len(),
            buf: String::new(),
            row: 1,
            col: 1,
        }
    }

    pub fn next_token(&mut self) -> Token {
        let start_chars = self.chars.clone();
        let Some(next_char) = self.next_char() else {
            return Token::new(TokenKind::EOF, 0, Span::default());
        };

        use TokenKind::*;
        let kind = match next_char {
            ';' => self.consume_line_comment(),
            '\'' | '`' => Backquote,
            '&' => And,
            '=' => Eq,
            '<' => OpenAngleBracket,
            '>' => CloseAngleBracket,
            '(' => OpenParen,
            ')' => CloseParen,
            ',' => Comma,

            c if is_whitespace(c) => self.consume_whitespace(),
            c if is_keyword_prefix(c) => self.consume_keyword(c),
            c if is_string_literal(c) => self.consume_string_literal(c),
            _c => self.consume_ident(),
        };

        let len = self.pos_within_cursor();
        let (span, advance_rows, col) = Span::make_span(self.row, self.col, start_chars, len);

        self.reset_span(advance_rows, col);
        self.reset_pos_within_cursor();

        Token { kind, len, span }
    }

    fn reset_span(&mut self, advance_rows: usize, col: usize) {
        self.row += advance_rows;
        self.col = col;
    }

    fn consume_keyword(&mut self, start: char) -> TokenKind {
        self.buf.clear();
        self.buf.push(start);
        loop {
            if is_end_ident(self.peak()) {
                break;
            }
            let Some(c) = self.next_char() else {
                break;
            };
            self.buf.push(c);
        }
        match self.buf.as_str() {
            s if is_keyword(s) => TokenKind::Keyword,
            _ => TokenKind::Ident,
        }
    }

    fn consume_line_comment(&mut self) -> TokenKind {
        loop {
            self.next_char();
            if self.peak() == '\n' {
                break;
            }
        }
        TokenKind::LineComment
    }

    fn consume_string_literal(&mut self, start: char) -> TokenKind {
        loop {
            let c = self.peak();

            self.next_char();
            if start == c {
                return TokenKind::Literal;
            }
        }
    }

    fn consume_ident(&mut self) -> TokenKind {
        loop {
            let c = self.peak();

            if is_end_ident(c) {
                return TokenKind::Ident;
            }

            self.next_char();
        }
    }

    fn consume_whitespace(&mut self) -> TokenKind {
        loop {
            match self.peak() {
                '\n' => {
                    self.next_char();
                }
                c if is_whitespace(c) => {
                    self.next_char();
                }
                _ => return TokenKind::Whitespace,
            }
        }
    }

    fn peak(&self) -> char {
        let mut chars = self.chars.clone();
        chars.next().unwrap_or('\0')
    }

    fn pos_within_cursor(&self) -> usize {
        self.len_remaining - self.chars.as_str().len()
    }

    fn reset_pos_within_cursor(&mut self) {
        self.len_remaining = self.chars.as_str().len();
    }

    fn next_char(&mut self) -> Option<char> {
        self.chars.next()
    }
}

impl Token {
    pub fn as_str<'a>(&self, src: &'a str) -> &'a str {
        if let TokenKind::EOF = self.kind {
            return "";
        }

        let find_idx = |r, c| {
            let mut col = 1;
            let mut row = 1;

            for (idx, char) in src.chars().enumerate() {
                if row == r && col == c {
                    return idx;
                }
                match char {
                    '\n' => {
                        col = 1;
                        row += 1;
                    }
                    _ => col += 1,
                }
            }
            src.len() - 1
        };

        let start = find_idx(self.span.start_row, self.span.start_col);
        let end = find_idx(self.span.end_row, self.span.end_col);

        &src[start..=end]
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}, {:02} => {:02}, {:02}: {:?}",
            self.span.start_row,
            self.span.start_col,
            self.span.end_row,
            self.span.end_col,
            self.kind,
        )
    }
}

impl Cursor<'_> {
    pub fn dbg_token(&self, token: &Token) {
        let mut stdout = stdout();
        writeln!(stdout, "{} {:?}", token, token.as_str(self.src)).unwrap();
        stdout.flush().unwrap();
    }
}
