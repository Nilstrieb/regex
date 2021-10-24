use crate::parse::Regex;

struct Transition {
    char: char,
}

struct Node {
    end: bool,
    transitions: Vec<Transition>,
}

struct RegexFsm {
    nodes: Vec<Node>,
}

/// Compiles the parsed Regex into a FSM
fn compile(regex: Regex) -> RegexFsm {
    let mut nodes = Vec::new();

    RegexFsm { nodes }
}
