
use std::str::Chars;
use std::iter::Peekable;

use crate::tree::{ Term, TermKind, Expr, ExprKind, Clause, atom };


#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Word(String),
    OpenBracket,
    CloseBracket,
    Comma,
    Horn,
    Period,

    Error(char),
}

pub struct Lexer<'input> {
    chars: Peekable<Chars<'input>>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    pub fn eat_whitespace(&mut self) {
        while self.chars.next_if(|ch| is_whitespace(*ch)).is_some() { }
    }

    pub fn get_string(&mut self, first: char, f: impl Fn(char) -> bool) -> String {
        let mut text = first.to_string();
        loop {
            if let Some(ch) = self.chars.next_if(|ch| f(*ch)) {
                text.push(ch);
            } else {
                break;
            }
        }
        text
    }

    pub fn get_token(&mut self) -> Option<Token> {
        self.eat_whitespace();

        match self.chars.next()? {
            ch if is_word(ch) => Some(Token::Word(self.get_string(ch, is_word))),
            '(' => Some(Token::OpenBracket),
            ')' => Some(Token::CloseBracket),
            ',' => Some(Token::Comma),
            '.' => Some(Token::Period),
            ':' => {
                match self.chars.next() {
                    Some(ch) if ch == '-' => Some(Token::Horn),
                    Some(ch) => Some(Token::Error(ch)),
                    None => Some(Token::Error(' ')),
                }
            },
            ch @ _ => Some(Token::Error(ch)),
        }
    }
}

fn is_whitespace(ch: char) -> bool {
    ch == ' ' || ch == '\n' || ch == '\t'
}

fn is_word(ch: char) -> bool {
    (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || (ch >= '0' && ch <= '9') || (ch == '_')
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_token()
    }
}




#[derive(Clone, Debug)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedToken(Token),
}

#[must_use]
#[inline(always)]
fn expect_next(input: &mut Peekable<Lexer>) -> Result<Token, ParseError> {
    input.next().ok_or(ParseError::UnexpectedEof)
}

#[inline]
#[must_use]
fn expect_token(input: &mut Peekable<Lexer>, token: Token) -> Result<(), ParseError> {
    let next = expect_next(input)?;
    match next == token {
        true => Ok(()),
        false => Err(ParseError::UnexpectedToken(next)),
    }
}

fn parse_atom_or_variable(name: String) -> Result<Term, ParseError> {
    match name.chars().nth(0) {
        Some(ch) if ch >= 'A' && ch <= 'Z' =>
            Ok(Box::new(TermKind::Var(name))),
        _ =>
            Ok(Box::new(TermKind::Atom(name))),
    }
}

fn parse_compound(input: &mut Peekable<Lexer>, name: String) -> Result<Term, ParseError> {
    match expect_next(input)? {
        Token::OpenBracket => {
            let args = parse_comma_separated(input)?;
            expect_token(input, Token::CloseBracket)?;
            Ok(Box::new(TermKind::Compound(name, args)))
        },
        token => Err(ParseError::UnexpectedToken(token)),
    }
}

fn parse_comma_separated(input: &mut Peekable<Lexer>) -> Result<Vec<Term>, ParseError> {
    let mut list = vec!();

    loop {
        list.push(parse_term(input)?);
        match input.peek() {
            Some(Token::Comma) => { input.next(); },
            _ => { break; },
        }
    }

    Ok(list)
}

fn parse_term(input: &mut Peekable<Lexer>) -> Result<Term, ParseError> {
    match expect_next(input)? {
        Token::Word(name) => {
            match input.peek() {
                Some(Token::OpenBracket) =>
                    parse_compound(input, name),
                _ =>
                    parse_atom_or_variable(name),
            }
        },
        token => Err(ParseError::UnexpectedToken(token)),
    }
}

fn parse_expression(input: &mut Peekable<Lexer>) -> Result<Expr, ParseError> {
    let term = Box::new(ExprKind::Term(parse_term(input)?));
    match input.peek() {
        Some(Token::Comma) => {
            input.next();
            Ok(Box::new(ExprKind::Conjunct(term, parse_expression(input)?)))
        },
        _ => Ok(term),
    }
}

fn parse_clause(input: &mut Peekable<Lexer>) -> Result<Clause, ParseError> {
    let term = parse_term(input)?;

    match expect_next(input)? {
        Token::Period => Ok(Clause::Fact(term)),
        Token::Horn => {
            let expr = parse_expression(input)?;
            expect_token(input, Token::Period)?;
            Ok(Clause::Rule(term, expr))
        },
        _ => Err(ParseError::UnexpectedEof),
    }
}

pub fn parse(text: &str) -> Result<Vec<Clause>, ParseError> {
    let mut input = Lexer::new(text).peekable();

    let mut clauses = vec!();
    loop {
        if input.peek().is_none() {
            break;
        }

        let clause = parse_clause(&mut input)?;
        clauses.push(clause);
    }

    Ok(clauses)
}
