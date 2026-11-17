use super::{Dots, Receiver};
use crate::error::{Position, Result};
use crate::lang::ast::lexer::tokenize;
use crate::lang::ast::parser::Parser;
use crate::lang::ast::{BinaryOp, Expr};

fn probe(source: &str, receiver: Receiver) -> Result<Expr> {
    let mut parser = Parser {
        dots: Dots::Receivers,
        receiver,
        ..Parser::new(tokenize(source).unwrap())
    };
    parser.accepted(Parser::expression)
}

fn this(property: &str) -> Expr {
    Expr::Member {
        object: Box::new(Expr::This),
        property: property.to_string(),
    }
}

#[test]
fn probes_of_accepted_parses_read_nothing_and_probes_of_the_final_parse_read_instances() {
    assert_eq!(Dots::Accepted.probed(), Dots::Accepted);
    assert_eq!(Dots::Receivers.probed(), Dots::Instances);
    assert_eq!(Dots::Instances.probed(), Dots::Instances);
}

#[test]
fn a_probe_reads_the_instance_and_no_assignment_receiver() {
    let read = probe("(.w + 1)", Receiver::Instance).unwrap();
    assert_eq!(
        read,
        Expr::Binary {
            operator: BinaryOp::Add,
            left: Box::new(this("w")),
            right: Box::new(Expr::Number(1.0)),
        }
    );
    let path = Receiver::Path(Expr::Identifier("o".to_string()));
    let error = probe("(.w + 1)", path).unwrap_err();
    assert_eq!(
        (error.message(), error.position()),
        ("Unexpected '.'".to_string(), Some(Position::new(1, 3)))
    );
    let variable = probe("(x = .w)", Receiver::Instance).unwrap();
    assert_eq!(
        variable,
        Expr::Assign {
            target: Box::new(Expr::Identifier("x".to_string())),
            value: Box::new(this("w")),
        }
    );
    let error = probe("(o.p = .w)", Receiver::Instance).unwrap_err();
    assert_eq!(error.position(), Some(Position::new(1, 9)));
    assert_eq!(error.message(), "Unexpected '.'");
}

#[test]
fn a_probe_keeps_the_error_of_an_assignment_target_that_starts_with_a_read() {
    let error = probe("(.w = 1)", Receiver::Instance).unwrap_err();
    assert_eq!(
        (error.message(), error.position()),
        ("Unexpected '.'".to_string(), Some(Position::new(1, 3)))
    );
}
