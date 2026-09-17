/*
 * Copyright (c) 2026 acctress.
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    IntegerLiteral(i64),
    FloatingPointLiteral(f64),
    BooleanLiteral(bool),
    CharacterLiteral(char),
    StringLiteral(String),
    NullLiteral,
    Identifier(String),
    Abstract, Assert, Break, Case, Catch, Class, Const, Continue,
    Default, Do, Else, Enum, Extends, Final, Finally, For,
    Goto, If, Implements, Import, Instanceof, Interface,
    Native, New, Package, Private, Protected, Public,
    Return, Static, Strictfp, Super, Switch, Synchronized,
    This, Throw, Throws, Transient, Try, Void, Volatile, While,
    Boolean, Byte, Char, Double, Float, Int, Long, Short,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Semicolon, Comma, Dot, Ellipsis, At, ColonColon,
    Assign, Plus, Minus, Star, Slash, Percent, Amp, Pipe,
    Caret, Tilde, Bang, Shl, Shr, UShr, Lt, Gt, LtEq, GtEq,
    EqEq, BangEq, AmpAmp, PipePipe, Question, Colon,
    Arrow, PlusPlus, MinusMinus, PlusAssign, MinusAssign,
    StarAssign, SlashAssign, PercentAssign, AmpAssign,
    PipeAssign, CaretAssign, ShlAssign, ShrAssign, UShrAssign,
    LineComment(String), BlockComment(String),
    Eof,
}

pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        loop {
            self.skip_ws();
            if !self.not_eof() {
                return None;
            }

            let cur = self.current();
            if cur == b'/' {
                match self.peek() {
                    Some(b'/') => { self.line_comment(); continue; }
                    Some(b'*') => { self.block_comment(); continue; }
                    _ => {}
                }
            }

            return Some(match cur {
                b'"' => self.string_literal(),
                b'\'' => self.char_literal(),
                b'0'..=b'9' => self.number_literal(),
                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => self.identifier_keyword(),
                b'(' => { self.advance(); Token::LParen }
                b')' => { self.advance(); Token::RParen }
                b'{' => { self.advance(); Token::LBrace }
                b'}' => { self.advance(); Token::RBrace }
                b'[' => { self.advance(); Token::LBracket }
                b']' => { self.advance(); Token::RBracket }
                b';' => { self.advance(); Token::Semicolon }
                b',' => { self.advance(); Token::Comma }
                b'@' => { self.advance(); Token::At }
                b'.' => {
                    if self.peek() == Some(b'.') && self.peek2() == Some(b'.') {
                        self.advance(); self.advance(); self.advance();
                        Token::Ellipsis
                    } else {
                        self.advance();
                        Token::Dot
                    }
                }
                b':' => {
                    self.advance();
                    if self.current_opt() == Some(b':') { self.advance(); Token::ColonColon }
                    else { Token::Colon }
                }
                b'=' => {
                    self.advance();
                    if self.current_opt() == Some(b'=') { self.advance(); Token::EqEq }
                    else { Token::Assign }
                }
                b'!' => {
                    self.advance();
                    if self.current_opt() == Some(b'=') { self.advance(); Token::BangEq }
                    else { Token::Bang }
                }
                b'+' => {
                    self.advance();
                    match self.current_opt() {
                        Some(b'+') => { self.advance(); Token::PlusPlus }
                        Some(b'=') => { self.advance(); Token::PlusAssign }
                        _ => Token::Plus
                    }
                }
                b'-' => {
                    self.advance();
                    match self.current_opt() {
                        Some(b'-') => { self.advance(); Token::MinusMinus }
                        Some(b'=') => { self.advance(); Token::MinusAssign }
                        Some(b'>') => { self.advance(); Token::Arrow }
                        _ => Token::Minus
                    }
                }
                b'*' => {
                    self.advance();
                    if self.current_opt() == Some(b'=') { self.advance(); Token::StarAssign }
                    else { Token::Star }
                }
                b'/' => {
                    self.advance();
                    if self.current_opt() == Some(b'=') { self.advance(); Token::SlashAssign }
                    else { Token::Slash }
                }
                b'%' => {
                    self.advance();
                    if self.current_opt() == Some(b'=') { self.advance(); Token::PercentAssign }
                    else { Token::Percent }
                }
                b'&' => {
                    self.advance();
                    match self.current_opt() {
                        Some(b'&') => { self.advance(); Token::AmpAmp }
                        Some(b'=') => { self.advance(); Token::AmpAssign }
                        _ => Token::Amp
                    }
                }
                b'|' => {
                    self.advance();
                    match self.current_opt() {
                        Some(b'|') => { self.advance(); Token::PipePipe }
                        Some(b'=') => { self.advance(); Token::PipeAssign }
                        _ => Token::Pipe
                    }
                }
                b'^' => {
                    self.advance();
                    if self.current_opt() == Some(b'=') { self.advance(); Token::CaretAssign }
                    else { Token::Caret }
                }
                b'<' => {
                    self.advance();
                    match self.current_opt() {
                        Some(b'=') => { self.advance(); Token::LtEq }
                        Some(b'<') => {
                            self.advance();
                            if self.current_opt() == Some(b'=') { self.advance(); Token::ShlAssign }
                            else { Token::Shl }
                        }
                        _ => Token::Lt
                    }
                }
                b'>' => {
                    self.advance();
                    if self.current_opt() == Some(b'=') { self.advance(); Token::GtEq }
                    else { Token::Gt }
                }
                b'~' => { self.advance(); Token::Tilde }
                b'?' => { self.advance(); Token::Question }
                _ => panic!("unexpected character: {:?}", cur as char),
            });
        }
    }

    pub fn all(&mut self) -> Vec<Token> {
        std::iter::from_fn(|| self.next()).collect()
    }

    fn current(&self) -> u8 {
        self.source.as_bytes()[self.pos]
    }

    fn advance(&mut self) {
        if self.not_eof() {
            self.pos += 1;
        }
    }

    fn skip_ws(&mut self) {
        while self.not_eof() && (self.current() as char).is_ascii_whitespace() {
            self.advance();
        }
    }

    fn not_eof(&self) -> bool {
        self.pos < self.source.len()
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos + 1).copied()
    }

    fn peek2(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos + 2).copied()
    }

    fn current_opt(&self) -> Option<u8> {
        if self.not_eof() { Some(self.current()) } else { None }
    }

    fn line_comment(&mut self) {
        while self.not_eof() && self.current() != b'\n' {
            self.advance();
        }
    }

    fn block_comment(&mut self) {
        self.advance(); self.advance();
        while self.not_eof() {
            if self.current() == b'*' && self.peek() == Some(b'/') {
                self.advance(); self.advance();
                return;
            }
            self.advance();
        }

        panic!("unterminated block comment");
    }

    fn identifier_keyword(&mut self) -> Token {
        let start = self.pos;
        while self.not_eof() && {
            let c = self.current();
            c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
        } {
            self.advance();
        }

        match &self.source[start..self.pos] {
            "abstract"     => Token::Abstract,
            "assert"       => Token::Assert,
            "boolean"      => Token::Boolean,
            "break"        => Token::Break,
            "byte"         => Token::Byte,
            "case"         => Token::Case,
            "catch"        => Token::Catch,
            "char"         => Token::Char,
            "class"        => Token::Class,
            "const"        => Token::Const,
            "continue"     => Token::Continue,
            "default"      => Token::Default,
            "do"           => Token::Do,
            "double"       => Token::Double,
            "else"         => Token::Else,
            "enum"         => Token::Enum,
            "extends"      => Token::Extends,
            "false"        => Token::BooleanLiteral(false),
            "final"        => Token::Final,
            "finally"      => Token::Finally,
            "float"        => Token::Float,
            "for"          => Token::For,
            "goto"         => Token::Goto,
            "if"           => Token::If,
            "implements"   => Token::Implements,
            "import"       => Token::Import,
            "instanceof"   => Token::Instanceof,
            "int"          => Token::Int,
            "interface"    => Token::Interface,
            "long"         => Token::Long,
            "native"       => Token::Native,
            "new"          => Token::New,
            "null"         => Token::NullLiteral,
            "package"      => Token::Package,
            "private"      => Token::Private,
            "protected"    => Token::Protected,
            "public"       => Token::Public,
            "return"       => Token::Return,
            "short"        => Token::Short,
            "static"       => Token::Static,
            "strictfp"     => Token::Strictfp,
            "super"        => Token::Super,
            "switch"       => Token::Switch,
            "synchronized" => Token::Synchronized,
            "this"         => Token::This,
            "throw"        => Token::Throw,
            "throws"       => Token::Throws,
            "transient"    => Token::Transient,
            "true"         => Token::BooleanLiteral(true),
            "try"          => Token::Try,
            "void"         => Token::Void,
            "volatile"     => Token::Volatile,
            "while"        => Token::While,
            s              => Token::Identifier(s.to_string()),
        }
    }

    fn string_literal(&mut self) -> Token {
        self.advance();

        let mut s = String::new();
        while self.not_eof() && self.current() != b'"' {
            s.push(self.escape_or_char());
        }

        self.advance();
        Token::StringLiteral(s)
    }

    fn char_literal(&mut self) -> Token {
        self.advance();

        let c = self.escape_or_char();
        assert_eq!(self.current(), b'\'', "unterminated char literal");

        self.advance();
        Token::CharacterLiteral(c)
    }

    fn escape_or_char(&mut self) -> char {
        let c = self.current() as char;
        self.advance();
        if c != '\\' { return c; }
        let esc = self.current() as char;
        self.advance();
        match esc {
            'n'  => '\n', 't'  => '\t', 'r'  => '\r',
            '\\' => '\\', '\'' => '\'', '"'  => '"',
            '0'  => '\0',
            'u'  => self.unicode_escape(),
            _    => panic!("unknown escape: \\{}", esc),
        }
    }

    fn unicode_escape(&mut self) -> char {
        let mut hex = String::with_capacity(4);
        for _ in 0..4 {
            hex.push(self.current() as char);
            self.advance();
        }

        char::from_u32(u32::from_str_radix(&hex, 16).unwrap()).unwrap()
    }

    fn number_literal(&mut self) -> Token {
        let start = self.pos;
        let mut is_float = false;

        if self.current() == b'0' {
            match self.peek() {
                Some(b'x') | Some(b'X') => return self.hex_literal(),
                Some(b'b') | Some(b'B') => return self.binary_literal(),
                _ => {}
            }
        }

        while self.not_eof() && (self.current().is_ascii_digit() || self.current() == b'_') {
            self.advance();
        }

        if self.not_eof() && (self.current() == b'.' || self.current() == b'e' || self.current() == b'E') {
            is_float = true;

            if self.current() == b'.' { self.advance(); }
            while self.not_eof() && self.current().is_ascii_digit() { self.advance(); }

            if self.not_eof() && (self.current() == b'e' || self.current() == b'E') {
                self.advance();
                if self.not_eof() && (self.current() == b'+' || self.current() == b'-') { self.advance(); }
                while self.not_eof() && self.current().is_ascii_digit() { self.advance(); }
            }
        }

        if self.not_eof() {
            match self.current() {
                b'L' | b'l' => { self.advance(); }
                b'f' | b'F' | b'd' | b'D' => { self.advance(); is_float = true; }
                _ => {}
            }
        }

        let raw: String = self.source[start..self.pos].chars().filter(|&c| c != '_').collect();

        if is_float {
            Token::FloatingPointLiteral(raw.trim_end_matches(|c| c == 'f' || c == 'F' || c == 'd' || c == 'D').parse().unwrap())
        } else {
            Token::IntegerLiteral(raw.trim_end_matches(|c| c == 'l' || c == 'L').parse().unwrap())
        }
    }

    fn hex_literal(&mut self) -> Token {
        self.advance(); self.advance();

        let start = self.pos;
        while self.not_eof() && (self.current().is_ascii_hexdigit() || self.current() == b'_') {
            self.advance();
        }

        let suffix = self.current_opt();
        if suffix == Some(b'L') || suffix == Some(b'l') { self.advance(); }

        let raw: String = self.source[start..self.pos].chars().filter(|&c| c != '_').collect();
        let raw = raw.trim_end_matches(|c| c == 'l' || c == 'L');

        Token::IntegerLiteral(i64::from_str_radix(raw, 16).unwrap())
    }

    fn binary_literal(&mut self) -> Token {
        self.advance(); self.advance();

        let start = self.pos;
        while self.not_eof() && (self.current() == b'0' || self.current() == b'1' || self.current() == b'_') {
            self.advance();
        }

        let suffix = self.current_opt();
        if suffix == Some(b'L') || suffix == Some(b'l') { self.advance(); }

        let raw: String = self.source[start..self.pos].chars().filter(|&c| c != '_').collect();
        let raw = raw.trim_end_matches(|c| c == 'l' || c == 'L');

        Token::IntegerLiteral(i64::from_str_radix(raw, 2).unwrap())
    }
}

