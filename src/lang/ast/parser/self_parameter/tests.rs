use super::{ReceiverMethod, Selves};
use crate::lang::ast::lexer::tokenize;
use crate::lang::ast::parser::Parser;

fn mentions(source: &str) -> bool {
    Parser::new(tokenize(source).unwrap()).mentions_self_parameters()
}

fn dropped(source: &str) -> Parser {
    Parser {
        selves: Selves::Dropped,
        ..Parser::new(tokenize(source).unwrap())
    }
}

#[test]
fn the_gate_holds_a_receiver_window_or_a_template_that_may_hold_one() {
    for source in [
        "def m(self)",
        "def m(this)",
        "def m(self, a)",
        "def m(this,)",
        "class P:\n    def m(self) => 1",
        "x = 1\ny = `${def m(self)}`",
        "`${function() { def m(this) }}`",
        "`def self`",
        "`selfdef`",
        "`this def`",
    ] {
        assert!(mentions(source), "{source}");
    }
    for source in [
        "x = )",
        "def m(a, self)",
        "def m(self.x)",
        "def m(self: P)",
        "def m(self = 1)",
        "def m(...self)",
        "def(self)",
        "def 5(self)",
        "def this(self)",
        "def m[self, a]",
        "class m(self)",
        "def m self",
        "def m(self",
        "function m(self)",
        "f(self)",
        "m(self, a)",
        "`${def m(a)}`",
        "`${self.x}`",
        "`def`",
        "`self`",
        "x = 1",
    ] {
        assert!(!mentions(source), "{source}");
    }
}

#[test]
fn receiver_methods_are_read_only_while_receivers_are_dropped() {
    let accepted = Parser::new(tokenize("def m(self)").unwrap());
    assert_eq!(accepted.receiver_method(), None);
    for (source, method) in [
        ("def m(self)", Some(ReceiverMethod::Ordinary)),
        ("def m(this, a)", Some(ReceiverMethod::Ordinary)),
        ("def init(self)", Some(ReceiverMethod::Constructor)),
        ("def __init__(this, a)", Some(ReceiverMethod::Constructor)),
        ("def initial(self)", Some(ReceiverMethod::Ordinary)),
        ("def __init(self)", Some(ReceiverMethod::Ordinary)),
        ("def m(a, self)", None),
        ("def m(self.a)", None),
        ("function m(self)", None),
        ("x def m(self)", None),
    ] {
        assert_eq!(dropped(source).receiver_method(), method, "{source}");
    }
}
