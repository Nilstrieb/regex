mod compile;
mod parse;

pub fn no_unused_code(regex: &str) {
    let _ = parse::Parser::parse(regex);
}
