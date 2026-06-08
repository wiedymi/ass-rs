//! Behavioural tests for the position and movement [`TagHandler`] implementations.

use super::*;
use crate::plugin::{TagHandler, TagResult};

#[test]
fn position_handler_valid() {
    let handler = PositionTagHandler;
    assert_eq!(handler.process("100,200"), TagResult::Processed);
    assert_eq!(handler.process("0,0"), TagResult::Processed);
    assert_eq!(handler.process("-50,100"), TagResult::Processed);
    assert_eq!(handler.process("100.5,200.75"), TagResult::Processed);
    assert_eq!(handler.process(" 100 , 200 "), TagResult::Processed);
}

#[test]
fn position_handler_invalid() {
    let handler = PositionTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("100"), TagResult::Failed(_)));
    assert!(matches!(handler.process("100,"), TagResult::Failed(_)));
    assert!(matches!(handler.process(",200"), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc,200"), TagResult::Failed(_)));
    assert!(matches!(handler.process("100,abc"), TagResult::Failed(_)));
    assert!(matches!(
        handler.process("100,200,300"),
        TagResult::Failed(_)
    ));
}

#[test]
fn position_handler_validation() {
    let handler = PositionTagHandler;
    assert!(handler.validate("100,200"));
    assert!(handler.validate("-50,100"));
    assert!(handler.validate("100.5,200.75"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("100"));
    assert!(!handler.validate("abc,200"));
}

#[test]
fn move_handler_valid_4_args() {
    let handler = MoveTagHandler;
    assert_eq!(handler.process("0,0,100,100"), TagResult::Processed);
    assert_eq!(handler.process("-50,-50,50,50"), TagResult::Processed);
    assert_eq!(
        handler.process("100.5,200.5,300.5,400.5"),
        TagResult::Processed
    );
    assert_eq!(handler.process(" 0 , 0 , 100 , 100 "), TagResult::Processed);
}

#[test]
fn move_handler_valid_6_args() {
    let handler = MoveTagHandler;
    assert_eq!(handler.process("0,0,100,100,0,1000"), TagResult::Processed);
    assert_eq!(
        handler.process("-50,-50,50,50,500,1500"),
        TagResult::Processed
    );
    assert_eq!(
        handler.process("100.5,200.5,300.5,400.5,0.0,1000.0"),
        TagResult::Processed
    );
}

#[test]
fn move_handler_invalid() {
    let handler = MoveTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("100"), TagResult::Failed(_)));
    assert!(matches!(handler.process("100,200"), TagResult::Failed(_)));
    assert!(matches!(
        handler.process("100,200,300"),
        TagResult::Failed(_)
    ));
    assert!(matches!(
        handler.process("100,200,300,400,500"),
        TagResult::Failed(_)
    ));
    assert!(matches!(
        handler.process("100,200,300,400,500,600,700"),
        TagResult::Failed(_)
    ));
    assert!(matches!(
        handler.process("abc,200,300,400"),
        TagResult::Failed(_)
    ));
    assert!(matches!(
        handler.process("100,abc,300,400"),
        TagResult::Failed(_)
    ));
}

#[test]
fn move_handler_validation() {
    let handler = MoveTagHandler;
    assert!(handler.validate("0,0,100,100"));
    assert!(handler.validate("0,0,100,100,0,1000"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("100,200"));
    assert!(!handler.validate("100,200,300"));
    assert!(!handler.validate("100,200,300,400,500"));
    assert!(!handler.validate("abc,200,300,400"));
}

#[test]
fn handlers_have_correct_names() {
    assert_eq!(PositionTagHandler.name(), "pos");
    assert_eq!(MoveTagHandler.name(), "move");
}

#[test]
fn create_position_handlers_returns_all() {
    let handlers = create_position_handlers();
    assert_eq!(handlers.len(), 2);

    let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
    assert!(names.contains(&"pos"));
    assert!(names.contains(&"move"));
}

#[test]
fn position_edge_cases() {
    let handler = PositionTagHandler;

    // Large numbers
    assert_eq!(handler.process("999999,999999"), TagResult::Processed);

    // Very small decimals
    assert_eq!(handler.process("0.0001,0.0001"), TagResult::Processed);

    // Mixed signs
    assert_eq!(handler.process("+100,-200"), TagResult::Processed);
}

#[test]
fn move_edge_cases() {
    let handler = MoveTagHandler;

    // All zeros
    assert_eq!(handler.process("0,0,0,0"), TagResult::Processed);
    assert_eq!(handler.process("0,0,0,0,0,0"), TagResult::Processed);

    // Large numbers
    assert_eq!(
        handler.process("999999,999999,0,0,0,99999"),
        TagResult::Processed
    );

    // Negative times (valid in ASS)
    assert_eq!(
        handler.process("0,0,100,100,-500,1000"),
        TagResult::Processed
    );
}

#[test]
fn whitespace_handling() {
    let pos_handler = PositionTagHandler;
    let move_handler = MoveTagHandler;

    // Various whitespace
    assert_eq!(pos_handler.process("  100  ,  200  "), TagResult::Processed);
    assert_eq!(
        move_handler.process("  0  ,  0  ,  100  ,  100  "),
        TagResult::Processed
    );
    assert_eq!(
        move_handler.process("  0  ,  0  ,  100  ,  100  ,  0  ,  1000  "),
        TagResult::Processed
    );

    // Tabs and newlines should be trimmed
    assert_eq!(pos_handler.process("\t100\t,\t200\t"), TagResult::Processed);
}
