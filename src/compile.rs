//!
//! Compiles the regex AST into a finite state machine
//!
//! The basic idea is to return the index and char transition to the new node, which will then
//! be put into the transitions of the current one
//!
//! Example regex: /a|b*c/
//!
//! Using {x} to reference items in the text below
//!
//! Has the AST
//! ```txt
//! Choice {1}
//!  |-- Char(a)
//!  |-- Seq {4}
//!      | -- Repeat(Char(b)) {5}
//!      | -- Char(c) {6}
//! ```
//! Should compile into the following state machine
//!
//! ```txt
//!          {3} a    {2}
//!          /-----(())
//! ( START )
//!         \------()-----(())
//!            b   /\   c
//!             b |_|
//! ```
//!
//! For that, we compile each node individually
//!
//! Compiling the choice node {2} will get us all nodes and transitions we need
//! Compiling the first char node will add a single node {2}.
//! This compilation will then return the index to this node and also the char it needs,
//!  being the char that is contained in this char node. The choice node {2} will then
//!  add a transition {3} to the start node it created.
//! Note that the choice node {2} does not now that it is the start node.
//!
//! The same is being done for the second child of the choice, although it's a bit more
//!  complicated for that one.
//! First we compile the seq node {4}. This will directly lead to compiling it's two child nodes,
//!  ({5}, {6}).
//! Compiling the repeat node {5} returns it's index and also the char that leads to it.
//! The char that leads to a repeat node is the one it repeats.
//! For the char node {6}, it's very similar to the char node below the choice node {1}.

use crate::parse;
use crate::parse::Regex;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Transition {
    target_node: usize,
    condition: TransitionType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TransitionType {
    Range(Range<char>),
    Primitive(Primitive),
    Char(char),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Primitive {
    Word,
    Digit,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Node {
    end: bool,
    transitions: Vec<Transition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegexFsm {
    nodes: Vec<Node>,
}

#[derive(Debug, Default)]
struct Compiler {
    nodes: Vec<Node>,
}

/// Compiles the parsed Regex into a FSM
fn compile(regex: &Regex) -> RegexFsm {
    let mut compiler = Compiler::default();

    compiler.compile(regex);

    RegexFsm {
        nodes: compiler.nodes,
    }
}

impl Compiler {
    /// Pushes a dummy node into the internal buffer and returns a new one that can be modified.
    /// After the node is processed, it should replace the dummy node
    fn new_node(&mut self) -> (Node, usize) {
        self.nodes.push(Node::default());
        let node = Node::default();
        (node, self.nodes.len() - 1)
    }

    /// Compiles a regex into the compiler and returns the index of the start node
    fn compile(&mut self, regex: &Regex) -> Transition {
        let (mut this_node, this_node_index) = self.new_node();

        let condition_to_this = match regex {
            Regex::Choice(a, b) => {
                let a_trans = self.compile(&a);
                let b_trans = self.compile(&b);
                this_node.transitions.push(a_trans);
                this_node.transitions.push(b_trans);
                todo!()
            }
            Regex::Sequence(_) => {
                todo!()
            }
            Regex::Repetition(_) => {
                todo!()
            }
            Regex::Set(_) => {
                todo!()
            }
            Regex::Range(_) => {
                todo!()
            }
            Regex::Primitive(primitive) => TransitionType::Primitive(match primitive {
                parse::Primitive::Word => Primitive::Word,
                parse::Primitive::Digit => Primitive::Digit,
            }),
            Regex::Char(char) => TransitionType::Char(*char),
        };
        std::mem::replace(self.nodes.get_mut(this_node_index).unwrap(), this_node);
        Transition {
            target_node: this_node_index,
            condition: condition_to_this,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compile::{Node, RegexFsm, Transition, TransitionType};
    use crate::parse::Regex;

    ///
    /// regex: /🌈/
    /// fsm:  () --🌈-- (())
    #[test]
    fn single_char() {
        let ast = Regex::Char('🌈');
        let fsm = super::compile(&ast);
        assert_eq!(
            fsm,
            RegexFsm {
                nodes: vec![
                    Node {
                        end: false,
                        transitions: vec![Transition {
                            target_node: 1,
                            condition: TransitionType::Char('🌈')
                        }]
                    },
                    Node {
                        end: true,
                        transitions: vec![]
                    }
                ]
            }
        )
    }
}
