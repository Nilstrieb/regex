//!
//!
//! Inspired by [Matt Mights article](https://matt.might.net/articles/parsing-regex-with-recursive-descent/)
//!
//! Parses the regular expression using the following grammar
//! ```txt
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
//!         | '[' { <set-elem> } ']'
//!
//! <set-elem> ::= <char>
//!             | <range>
//!
//! <range> ::= <char> '-' <char>
//! ```

use std::iter::Peekable;
use std::ops::Range;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Regex {
    Choice(Box<Regex>, Box<Regex>),
    Sequence(Vec<Regex>),
    Repetition(Box<Regex>),
    Set(Vec<Regex>),
    Range(Range<char>),
    Primitive(Primitive),
    Char(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Primitive {
    Word,
    Digit,
}

#[derive(Debug)]
pub struct Parser<'a> {
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

    #[must_use]
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
            let _ = self.next();
            let rhs = self.regex()?;
            Ok(Regex::Choice(Box::new(term), Box::new(rhs)))
        } else {
            Ok(term)
        }
    }

    /// a term is a sequence of factors
    fn term(&mut self) -> RegexResult {
        let mut sequence = Vec::new();

        loop {
            if let None | Some(')') | Some('|') = self.peek() {
                break;
            }
            let next_factor = self.factor()?;
            sequence.push(next_factor);
        }

        Ok(Regex::Sequence(sequence))
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
                let esc = self.next().ok_or(())?;
                Ok(Regex::Primitive(match esc {
                    'w' => Primitive::Word,
                    'd' => Primitive::Digit,
                    _ => return Err(()),
                }))
            }
            Some('[') => {
                let _ = self.next();
                let mut elems = Vec::new();
                while self.peek() != Some(']') {
                    elems.push(self.set_elem()?);
                }
                let _ = self.next();
                Ok(Regex::Set(elems))
            }
            Some(char) => {
                let _ = self.next();
                Ok(Regex::Char(char))
            }
            None => Err(()),
        }
    }

    fn set_elem(&mut self) -> RegexResult {
        let first_char = self.next().ok_or(())?;

        if let Some('-') = self.peek() {
            let _ = self.next();
            let second_char = self.next().ok_or(())?;
            Ok(Regex::Range(first_char..second_char))
        } else {
            Ok(Regex::Char(first_char))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse::{Parser, Regex, Regex::*};

    fn char_seq(char: char) -> Regex {
        Sequence(vec![Char(char)])
    }

    fn box_char_seq(char: char) -> Box<Regex> {
        Box::new(Sequence(vec![Char(char)]))
    }

    #[test]
    fn simple_choice() {
        let regex = "a|b";
        let parsed = Parser::parse(regex).unwrap();
        assert_eq!(parsed, Choice(box_char_seq('a'), box_char_seq('b')))
    }

    #[test]
    fn repetition() {
        let regex = "a*";
        let parsed = Parser::parse(regex).unwrap();
        assert_eq!(parsed, Sequence(vec![Repetition(Box::new(Char('a')))]))
    }

    #[test]
    fn primitives() {
        let regex = "\\w\\d";
        let parsed = Parser::parse(regex).unwrap();
        assert_eq!(
            parsed,
            Sequence(vec![
                Primitive(super::Primitive::Word),
                Primitive(super::Primitive::Digit)
            ])
        )
    }

    #[test]
    fn groups() {
        let regex = "(a)(bc)";
        let parsed = Parser::parse(regex).unwrap();
        assert_eq!(
            parsed,
            Sequence(vec![char_seq('a'), Sequence(vec![Char('b'), Char('c')])])
        )
    }

    #[test]
    fn set() {
        let regex = "[ab]";
        let parsed = Parser::parse(regex).unwrap();
        assert_eq!(
            parsed,
            Sequence(vec![Regex::Set(vec![Regex::Char('a'), Regex::Char('b')])])
        )
    }

    #[test]
    fn set_range() {
        let regex = "[a-zA-Z]";
        let parsed = Parser::parse(regex).unwrap();
        assert_eq!(
            parsed,
            Sequence(vec![Regex::Set(vec![
                Regex::Range('a'..'z'),
                Regex::Range('A'..'Z')
            ])])
        )
    }
}
