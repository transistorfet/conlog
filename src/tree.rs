
use std::fmt;
use std::fmt::Debug;
use std::convert::From;


#[derive(Clone, Debug, PartialEq)]
pub enum TermKind {
    EmptyList,
    Var(String),
    Atom(String),
    Compound(String, Vec<Term>),
    List(Term, Term),
}

pub type Term = Box<TermKind>;


#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Term(Term),
    Conjunct(Expr, Expr),
}

pub type Expr = Box<ExprKind>;


#[derive(Clone, Debug, PartialEq)]
pub enum Clause {
    // you could eliminate the fact special case by having a termkind True, which is the rhs of a rule (such that True is a special case rather than a normal Atom)
    Fact(Term),
    Rule(Term, Expr),
}


impl From<Term> for Expr {
    fn from(item: Term) -> Self {
        Box::new(ExprKind::Term(item))
    }
}


#[allow(dead_code)]
pub fn variable(name: &str) -> Term {
    Box::new(TermKind::Var(name.to_string()))
}

#[allow(dead_code)]
pub fn atom(name: &str) -> Term {
    Box::new(TermKind::Atom(name.to_string()))
}

#[allow(dead_code)]
pub fn compound(name: &str, args: Vec<Term>) -> Term {
    Box::new(TermKind::Compound(name.to_string(), args))
}

#[allow(dead_code)]
pub fn empty_list() -> Term {
    Box::new(TermKind::EmptyList)
}

#[allow(dead_code)]
pub fn cons_list(term: Term, tail: Term) -> Term {
    Box::new(TermKind::List(term, tail))
}

#[allow(dead_code)]
pub fn conjunct(expr1: impl Into<Expr>, expr2: impl Into<Expr>) -> Expr {
    Box::new(ExprKind::Conjunct(expr1.into(), expr2.into()))
}

#[allow(dead_code)]
pub fn fact(term: Term) -> Clause {
    Clause::Fact(term)
}

#[allow(dead_code)]
pub fn rule(lhs: Term, rhs: impl Into<Expr>) -> Clause {
    Clause::Rule(lhs, rhs.into())
}


impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &**self {
            TermKind::EmptyList => write!(f, "[]"),
            TermKind::Atom(s) => write!(f, "{}", s),
            TermKind::Var(s) => write!(f, "{}", s),
            TermKind::Compound(s, args) => {
                let args = args.iter().map(|arg| format!("{}", arg)).collect::<Vec<String>>().join(", ");
                write!(f, "{}({})", s, args)
            },
            TermKind::List(head, tail) => {
                let mut list = format!("{}", head);
                let mut last = tail;
                loop {
                    match &**last {
                        TermKind::EmptyList => break,
                        TermKind::List(head, tail) => {
                            list += &format!(", {}", head);
                            last = tail;
                        },
                        _ => {
                            // This should only appear once in a well-formed tree, but we allow it to loop so as not to hide a malformed tree
                            list += &format!("| {}", head);
                            last = tail;
                        },
                    }
                }
                write!(f, "[{}]", list)
            },
        }
    }
}

