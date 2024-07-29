use might::Parser;

fn main() {
    let input = include_str!("../example.mig");

    let tests = Parser::parse(input).unwrap();

    let res = tests.spawn();

    res.iter().for_each(|r| println!("{r}"))
}
