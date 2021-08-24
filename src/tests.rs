
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
        test([]).
        test([X|Xs]) :- test(Xs).
        ",
        "
        test([thing, stuff, cat]).
        ");

	assert_eq!(format!("{}", partial.result), "test([thing, stuff, cat])");
    }

    #[test]
    fn list_append_test() {
	let partial = solve_program_with_query("
        append([], Ys, Ys).
        append([X|Xs], Ys, [X|Zs]) :- append(Xs, Ys, Zs).
        ",
        "
        append([thing, stuff, cat], [more, cat, stuff], Zs).
        ");

	assert_eq!(format!("{}", partial.result), "append([thing, stuff, cat], [more, cat, stuff], [thing, stuff, cat, more, cat, stuff])");
    }

    #[test]
    fn list_member_true_test() {
	let partial = solve_program_with_query("
        member(X, [X|Xs]).
        member(X, [Y|Xs]) :- member(X, Xs).
        ",
        "
        member(cat, [thing, cat, stuff]).
        ");

	assert_eq!(format!("{}", partial.result), "member(cat, [thing, cat, stuff])");
    }

    #[test]
    fn list_delete_test() {
	let partial = solve_program_with_query("
        delete(X, [], []).
        delete(X, [X|Ys], Zs) :-  delete(X, Ys, Zs).
        delete(X, [Y|Ys], [Y|Zs]) :- X \\= Y, delete(X, Ys, Zs).
        ",
        "
        delete(cat, [cat, thing, stuff, stuff, cat], Ys).
        ");

	assert_eq!(format!("{}", partial.result), "delete(cat, [cat, thing, stuff, stuff, cat], [thing, stuff, stuff])");
    }

    #[test]
    fn list_reverse_test() {
	let partial = solve_program_with_query("
        reverse(X, Y) :- reverse(X, Y, []).
        reverse([], Z, Z).
        reverse([H|T], Z, Acc) :- reverse(T, Z, [H|Acc]).
        ",
        "
        reverse([cat, dog, bird], X).
        ");

	assert_eq!(format!("{}", partial.result), "reverse([cat, dog, bird], [bird, dog, cat])");
    }

    #[test]
    fn integer_highest_test() {
	let partial = solve_program_with_query("
        highest(X, [X|[]]).
        highest(X, [X|Xs]) :- highest(Y, Xs), X >= Y.
        highest(Y, [X|Xs]) :- highest(Y, Xs), X < Y.
        ",
        "
        highest(X, [1, 8, 904, 234, 42]).
        ");

	assert_eq!(format!("{}", partial.result), "highest(904, [1, 8, 904, 234, 42])");
    }

    #[test]
    fn integer_nth() {
	let partial = solve_program_with_query("
        nth([X|Xs], 0, X).
        nth([S|Xs], N, Y) :- M is N - 1, nth(Xs, M, Y). 
        ",
        "
        nth([1, 8, 904, 234, 42], 3, X).
        ");

	assert_eq!(format!("{}", partial.result), "nth([1, 8, 904, 234, 42], 3, 234)");
    }
}

