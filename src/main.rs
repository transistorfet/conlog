
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
    */


    let input = "
        [thing, stuff, cat].
        test([]).
        test([X|Xs]) :- test(Xs).
    ";

    let query_string = "
        test([thing, stuff, cat]).
    ";



    /*
    let clauses = vec![
        fact(atom("True")),
        fact(atom("False")),
        rule(atom("Pants"), atom("True")),
        rule(compound("Pants", vec!(variable("X"))), variable("X")),
    ];
    */
    /*
    let clauses = vec![
        fact(compound("female", vec!(atom("marge")))),
        fact(compound("female", vec!(atom("lisa")))),
        fact(compound("male", vec!(atom("homer")))),
        fact(compound("male", vec!(atom("bart")))),
        fact(compound("parent", vec!(atom("marge"), atom("bart")))),
        fact(compound("parent", vec!(atom("homer"), atom("bart")))),
        fact(compound("parent", vec!(atom("marge"), atom("lisa")))),
        fact(compound("parent", vec!(atom("homer"), atom("lisa")))),
        rule(compound("father", vec!(variable("X"), variable("Y"))), conjunct(compound("parent", vec!(variable("X"), variable("Y"))), compound("male", vec!(variable("X"))))),
    ];

    let mut query = Query::new(compound("father", vec!(variable("X"), atom("bart"))));
    */

    /*
    let clauses = vec![
        fact(atom("ActionA")),
        fact(atom("ActionB")),
        fact(atom("ActionC")),

        fact(atom("cond1")),
        fact(atom("cond2")),
        rule(atom("cond2"), conjunct(atom("cond1"), atom("cond3"))),
    ];
    */

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

