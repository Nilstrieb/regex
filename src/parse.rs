//!
//!
//! Inspired by [Matt Mights article](https://matt.might.net/articles/parsing-regex-with-recursive-descent/)
//!
//! Parses the regular expression using the following grammar
//! ```text
//! # e.g.   abc|c(de)*
//! <regex> ::= <term> '|' <regex>
//!          | term
//!
//! <term> ::= { <factor> }
//!
//! <factor> ::= <base> { '*' }
//!
//! <base> ::= <char>
//!         | '\' <char>
//!         | '(' <regex> ')'
//! ```

use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub enum Regex {
    Choice(Box<Regex>, Box<Regex>),
    Term(Vec<Regex>),
    Repetition(Box<Regex>),
    Primitive(char),
    Char(char),
}

#[derive(Debug)]
struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
}

type RegexResult = Result<Regex, ()>;

impl<'a> Parser<'a> {
    pub fn parse(regex: &'a str) -> Result<Regex, ()> {
        let chars = regex.chars();
        let mut parser = Self {
            chars: chars.peekable(),
        };
        parser.regex()
    }

    fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn expect(&mut self, c: char) {
        if self.peek() == Some(c) {
            let _ = self.next();
        } else {
            panic!("handle this better")
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().cloned()
    }

    // regex term types

    fn regex(&mut self) -> RegexResult {
        let term = self.term()?;
        if let Some('|') = self.peek() {
            let rhs = self.regex()?;
            Ok(Regex::Choice(Box::new(term), Box::new(rhs)))
        } else {
            Ok(term)
        }
    }

    fn term(&mut self) -> RegexResult {
        let mut factors = Vec::new();

        loop {
            match self.peek() {
                None | Some('(') | Some('|') => break,
                _ => {}
            }
            let next_factor = self.factor()?;
            factors.push(next_factor);
        }
        Ok(Regex::Term(factors))
    }

    fn factor(&mut self) -> RegexResult {
        let mut base = self.base()?;

        while let Some('*') = self.peek() {
            let _ = self.next();
            base = Regex::Repetition(Box::new(base));
        }

        Ok(base)
    }
    fn base(&mut self) -> RegexResult {
        match self.peek() {
            Some('(') => {
                let _ = self.next();
                let regex = self.regex()?;
                self.expect(')');
                Ok(regex)
            }
            Some('\\') => {
                let _ = self.next();
                let esc = self.next().ok_or_else(|| ())?;
                Ok(Regex::Primitive(esc))
            }
            Some(char) => {
                let _ = self.next();
                Ok(Regex::Char(char))
            }
            None => Err(()),
        }
    }
}

#[cfg(test)]
mod test {}
