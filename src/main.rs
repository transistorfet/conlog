
mod tree;
mod parser;
mod solver;

#[allow(unused_imports)]
use tree::{ TermKind, Clause, variable, atom, compound, conjunct, fact, rule };
use parser::{ parse };
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
    */

    let input = "
        not(X) :- X, !, fail.
        thing.
        has(thing) :- not(thing).
    ";

    let clauses = parse(input).unwrap();
    println!("{:?}", clauses);


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

    let db = Database::new(clauses);

    //let mut query = Query::new(compound("father", vec!(variable("X"), atom("bart"))));
    let query = Query::new(compound("has", vec!(atom("thing"))));
    let partial = query.solve(&db);

    println!("Result: {:?}", partial.map(|p| p.result));
}

