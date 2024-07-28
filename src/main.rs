use might::Parser;

fn main() {
    let input = include_str!("../example.mig");

    let tests = Parser::parse(input).unwrap();

    tests.spawn();
}
