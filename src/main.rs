
use std::io;
use std::env;
use std::fs::File;
use std::io::{ Read, Write };

mod misc;
mod tree;
mod parser;
mod solver;
mod tests;

#[allow(unused_imports)]
use tree::{ TermKind, Clause, variable, atom, compound, conjunct, fact, rule };
use parser::{ parse, parse_query };
use solver::{ Database, Query, Partial };

fn run_default() {
    /*
    let input = "
        female(marge).
        female(lise).
        male(homer).
        male(bart).
        parent(marge, bart).
        parent(marge, lisa).
        parent(homer, bart).
        parent(homer, lisa).
        father(X, Y) :- parent(X, Y), male(X).
    ";

    let query_string = "
        father(X, bart).
    ";
    */

    /*
    let input = "
        not(X) :- X, !, fail.
        thing.
        has(thing) :- not(thing).
    ";

    let input = "
        [thing, stuff, cat].
        test([]).
        test([X|Xs]) :- test(Xs).
    ";

    let query_string = "
        test([thing, stuff, cat]).
    ";
    */

    /*
    let input = "
        not(X) :- X, !, fail.
        delete(X, [], []).
        delete(X, [X|Ys], Zs) :- delete(X, Ys, Zs).
        delete(X, [Y|Ys], [Y|Zs]) :- not(equal(X, Y)), delete(X, Ys, Zs).
    ";

    let query_string = "
        delete(cat, [cat, thing, stuff, stuff, cat], Ys).
    ";
    */

    let input = "
        nth([X|Xs], 0, X).
        nth([S|Xs], N, Y) :- is(M, N - 1), nth(Xs, M, Y).
    ";

    let query_string = "
        nth([1, 2, 3, 4], 2, X).
    ";


    let clauses = parse(input).unwrap();
    println!("{:?}", clauses);
    let db = Database::new(clauses);

    let query_term = parse_query(query_string).unwrap();
    println!("{:?}", query_term);
    let query = Query::new(query_term);

    let mut at_rule = 0;
    for _ in 0..5 {
        let partial = query.solve_from(&db, at_rule);
        match partial {
            Some(partial) => {
                println!("Result: \x1b[32m{}\x1b[0m", partial.result);
                at_rule = partial.rule + 1;
            },
            None => {
                println!("Result: \x1b[31mfalse\x1b[0m");
                break;
            },
        }
    }
}

fn load_database(filename: &str) -> Database {
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let clauses = parse(&contents).unwrap();
    println!("{:?}", clauses);
    Database::new(clauses)
}

fn run_query(db: &Database, query: &str) -> Option<Partial> {
    let query_term = parse_query(query).ok()?;
    println!("{:?}", query_term);
    let query = Query::new(query_term);
    query.solve(db)
}

fn repl(db: Database) {
    loop {
        let mut input = String::new();
        io::stdout().write_all(b"?- ").unwrap();
        io::stdout().flush().unwrap();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                match run_query(&db, &input) {
                    Some(partial) =>
                        println!("Result: \x1b[32m{}\x1b[0m", partial.result),
                    None =>
                        println!("Error"),
                }
            }
            Err(err) => println!("IO Error: {:?}", err),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        run_default();
    } else {
        let db = load_database(&args[1]);
        repl(db);
    }
}

