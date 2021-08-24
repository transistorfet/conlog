 
Conlog
======

###### *Started June 2, 2021*

A simple prolog interpreter written in Rust with no external crate depedencies.
It currently supports predicates, integers and lists (but not strings), and
will parse some of the infix operators including +, -, =, \=, and 'is'.

To run the REPL:
```
cargo run -- inputfile.plg
```
where "inputfile.plg" has an initial set of facts and rules to populate the
database with.  It will then accept input at the `?- ` prompt.

Try running the `metro1.plg` file and at the prompt enter:
```
?- murderer(X).
```
You should see a trace of the deduction and the final result:
```
Result: murderer(sir_raymond)
```

Some tests are also included, which can be run using:
```
cargo test
```

