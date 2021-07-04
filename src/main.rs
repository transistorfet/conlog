
mod misc;
mod tree;
mod parser;
mod solver;
mod tests;

#[allow(unused_imports)]
use tree::{ TermKind, Clause, variable, atom, compound, conjunct, fact, rule };
use parser::{ parse, parse_query };
use solver::{ Database, Query };

fn main() {
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

    let input = "
        not(X) :- X, !, fail.
        delete(X, [], []).
        delete(X, [X|Ys], Zs) :- delete(X, Ys, Zs).
        delete(X, [Y|Ys], [Y|Zs]) :- not(equal(X, Y)), delete(X, Ys, Zs).
    ";

    let query_string = "
        delete(cat, [cat, thing, stuff, stuff, cat], Ys).
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

