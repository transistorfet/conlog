
mod solver;

use solver::{ TermKind, Clause, Database, Query, Partial, variable, atom, compound, conjunct, fact, rule };

fn main() {
    /*
    let clauses = vec![
        fact(atom("True")),
        fact(atom("False")),
        rule(atom("Pants"), atom("True")),
        rule(compound("Pants", vec!(variable("X"))), variable("X")),
    ];
    */
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

    let db = Database::new(clauses);

    let mut query = Query::new(compound("father", vec!(variable("X"), atom("bart"))));
    let partial = query.solve(&db);

    println!("Result: {:?}", partial.map(|p| p.result));
}

