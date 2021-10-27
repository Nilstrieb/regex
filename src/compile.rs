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
//!          {3}      {2}
//!          /--a--(())
//! ( START )
//!         \--b---()--c--(())
//!                /\   
//!               b_|
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
//!
//! Another example: /u(w|o)!/
//!
//! AST:  
//! ```txt
//! Seq
//!  |-- Char(u)
//!  |-- Choice
//!         |-- Char(w)
//!         |-- Char(o)
//!  |-- Char(!)
//! ```
//!
//! ```txt
//!
//!                 /-w--()--\
//! ( START )--u--()          |--!-(())
//!                 \-o--()--/
//!
//! ```
//!
//!
//! AST nodes will become transitions in the FSM  
//! FSM nodes are the connections in the AST
//!
//! This architecture mostly seems to work out, with the only problem currently being allocating nodes
//! this appears to be something not every kind of regex part does.
//!
//! A char will allocate the node for its transition.  
//! A seq won't to that, because the contents of the seq allocate everything, the seq is just a wrapper.  
//! Now the question is: is seq unique and should be special cased, or can something like it exist?
//!
//! Does choice allocate a node? No, it does not, it only branches. So allocating seems like something
//! that some kinds do, but not all of them.  
//!
//! So allocating is something that is not fundamental to the compilation, but handled by each node.

type NodeIndex = usize;

impl Compiler {
    /// This function takes the node index of the previous node, constructs a new one as the target,
    /// and then creates a transition from the previous to the new one, containing the condition
    /// of the AST node it is compiling.
    /// It returns
    fn compile(&mut self, regex: &Regex, node_before: NodeIndex) -> NodeIndex {
        match regex {
            Regex::Char(char) => self.allocating(node_before, |_, _| TransitionType::Char(*char)),
            Regex::Sequence(terms) => {
                if let Some(first) = terms.first() {
                    let trans_to_first = self.compile(first, 0);
                } else {
                    TransitionType::Always;
                };
                todo!()
            }
            Regex::Primitive(primitive) => self.allocating(node_before, |_, _| {
                TransitionType::Primitive(match primitive {
                    parse::Primitive::Word => Primitive::Word,
                    parse::Primitive::Digit => Primitive::Digit,
                })
            }),
            Regex::Choice(a, b) => {
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
        }
    }

    fn allocating<F: FnOnce(&mut Node, NodeIndex) -> TransitionType>(
        &mut self,
        node_before: NodeIndex,
        f: F,
    ) -> NodeIndex {
        let next_node_slot = self.reserve_node_slot();
        let mut next_node = Node::default();
        let this_condition = f(&mut next_node, next_node_slot);
        // fill the placeholder with the node we just created, forget the placeholder
        let _ = std::mem::replace(self.nodes.get_mut(next_node_slot).unwrap(), next_node);
        self.nodes
            .get_mut(node_before)
            .unwrap()
            .transitions
            .push(Transition {
                target_node: next_node_slot,
                condition: this_condition,
            });

        next_node_slot
    }
}

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
    Always,
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

    // reserve the start node
    compiler.reserve_node_slot();

    compiler.compile(regex, 0);

    RegexFsm {
        nodes: compiler.nodes,
    }
}

impl Compiler {
    /// Pushes a placeholder node into the internal buffer and returns it's index
    fn reserve_node_slot(&mut self) -> NodeIndex {
        self.nodes.push(Node::default());
        self.nodes.len() - 1
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
