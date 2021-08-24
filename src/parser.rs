
use std::str::Chars;
use std::iter::Peekable;

use crate::tree::{ Term, TermKind, Expr, ExprKind, Clause, empty_list, cons_list };

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Word(String),
    OpenBracket,
    CloseBracket,
    OpenSquare,
    CloseSquare,
    VerticalBar,
    Comma,
    Horn,
    Period,
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

    pub fn get_token(&mut self) -> Option<Token> {
        self.eat_whitespace();

        match self.chars.next()? {
            '%' => {
                // Ignore comment lines, which start with a '%' character
                while self.chars.next_if(|ch| *ch != '\n').is_some() { }
                self.get_token()
            },
            ch if is_word(ch) => Some(Token::Word(self.get_string(ch, is_word))),
            '(' => Some(Token::OpenBracket),
            ')' => Some(Token::CloseBracket),
            '[' => Some(Token::OpenSquare),
            ']' => Some(Token::CloseSquare),
            '|' => Some(Token::VerticalBar),
            '.' => Some(Token::Period),
            ',' => Some(Token::Comma),
            ch => {
                match self.get_string(ch, is_operator) {
                    op if op.as_str() == ":-" => Some(Token::Horn),
                    op => Some(Token::Word(op)),
                }
            },
        }
    }

    fn eat_whitespace(&mut self) {
        while self.chars.next_if(|ch| is_whitespace(*ch)).is_some() { }
    }

    fn get_string(&mut self, first: char, f: impl Fn(char) -> bool) -> String {
        let mut text = first.to_string();
        while let Some(ch) = self.chars.next_if(|ch| f(*ch)) {
            text.push(ch);
        }
        text
    }
}

fn is_whitespace(ch: char) -> bool {
    ch == ' ' || ch == '\n' || ch == '\t'
}

fn is_word(ch: char) -> bool {
    ('a'..='z').contains(&ch) || ('A'..='Z').contains(&ch) || ('0'..='0').contains(&ch) || (ch == '_')
}

fn is_operator(ch: char) -> bool {
    match ch {
        ';' | ':' | '=' | '>' | '<' | '+' | '-' | '*' | '\\' | '/' | '!' | '#' | '$' | '?' | '@' | '^' => true,
        _ => false,
    }
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

#[inline(always)]
fn expect_next(input: &mut Peekable<Lexer>) -> Result<Token, ParseError> {
    input.next().ok_or(ParseError::UnexpectedEof)
}

#[inline]
fn expect_token(input: &mut Peekable<Lexer>, token: Token) -> Result<(), ParseError> {
    let next = expect_next(input)?;
    match next == token {
        true => Ok(()),
        false => Err(ParseError::UnexpectedToken(next)),
    }
}

fn parse_number(name: String) -> Result<i64, ParseError> {
    let (num, _) = name.chars().rev().fold((0, 1), |acc, ch| {
        match ch {
            '-' => (-acc.0, acc.1),
            _ => ((ch.to_digit(10).unwrap() as i64 * acc.1) + acc.0, acc.1 * 10)
        }
    });
    Ok(num)
}

fn parse_atom_or_variable(name: String) -> Result<Term, ParseError> {
    match name.chars().next() {
        Some(ch) if ('0'..='9').contains(&ch) || ch == '-' => {
            Ok(Box::new(TermKind::Integer(parse_number(name)?)))
        },
        Some(ch) if ('A'..='Z').contains(&ch) =>
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

fn parse_list(input: &mut Peekable<Lexer>) -> Result<Term, ParseError> {
    if let Some(Token::CloseSquare) = input.peek() {
        input.next();
        return Ok(empty_list());
    }

    let mut new_list = vec!();
    let mut terms = empty_list();

    loop {
        new_list.push(parse_term(input)?);

        match expect_next(input)? {
            Token::Comma => { /* continue the loop */ },
            Token::CloseSquare => {
                break;
            },
            Token::VerticalBar => {
                terms = parse_term(input)?;
                expect_token(input, Token::CloseSquare)?;
                break;
            },
            token => return Err(ParseError::UnexpectedToken(token)),
        }
    }

    // Build the cons list from the vec of items
    for item in new_list.into_iter().rev() {
        terms = cons_list(item, terms);
    }

    Ok(terms)
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

const OPERATORS: [&str; 10] = [ ",", "=", "\\=", ">", ">=", "<", "<=", "+", "-", "is" ];

fn parse_term(input: &mut Peekable<Lexer>) -> Result<Term, ParseError> {
    let term = match expect_next(input)? {
        Token::Word(name) => {
            match input.peek() {
                Some(Token::OpenBracket) =>
                    parse_compound(input, name),
                _ =>
                    parse_atom_or_variable(name),
            }
        },
        Token::OpenSquare => {
            parse_list(input)
        },
        token => Err(ParseError::UnexpectedToken(token)),
    };

    match input.peek() {
        Some(Token::Word(name)) if OPERATORS.iter().any(|s| *s == name) => {
            let name = name.to_string();
            input.next();
            Ok(Box::new(TermKind::Compound(name, vec!(term?, parse_term(input)?))))
        },
        _ => term,
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

pub fn parse_query(text: &str) -> Result<Term, ParseError> {
    let mut input = Lexer::new(text).peekable();

    let term = parse_term(&mut input)?;

    match input.next() {
        Some(Token::Period) => Ok(term),
        Some(token) => Err(ParseError::UnexpectedToken(token)),
        None => Err(ParseError::UnexpectedEof),
    }
}

