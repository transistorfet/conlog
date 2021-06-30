
use crate::parser::{ parse, parse_query };
use crate::solver::{ Database, Query, Partial };

#[allow(dead_code)]
pub fn solve_program_with_query(program: &str, query: &str) -> Partial {
    let db = Database::new(parse(program).unwrap());
    let query = Query::new(parse_query(query).unwrap());
    query.solve(&db).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::tests::{ solve_program_with_query };

    #[test]
    fn basic_backtracking_test() {
	let partial = solve_program_with_query("
        female(marge).
        female(lise).
        male(homer).
        male(bart).
        parent(marge, bart).
        parent(marge, lisa).
        parent(homer, bart).
        parent(homer, lisa).
        father(X, Y) :- parent(X, Y), male(X).
    	",
        "
        father(X, bart).
    	");

	assert_eq!(format!("{}", partial.result), "father(homer, bart)");
    }

    #[test]
    fn basic_list_test() {
	let partial = solve_program_with_query("
        [thing, stuff, cat].
        test([]).
        test([X|Xs]) :- test(Xs).
    	",
        "
        test([thing, stuff, cat]).
    	");

	assert_eq!(format!("{}", partial.result), "test([thing, stuff, cat])");
    }
}

