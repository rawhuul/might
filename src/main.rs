use parser::MIG;

mod parser;

fn main() {
    let input = include_str!("../example.mig");

    _ = MIG::parse(input).unwrap();
}
