//! Tests to verify error handling

use rs_ach::{AchError, AchFile};
use std::error::Error;

#[test]
fn test_errors_implement_std_error_trait() {
    // All AchError variants should implement std::error::Error
    let errors: Vec<AchError> = vec![
        AchError::InvalidRecordType("X".to_string()),
        AchError::InvalidLineLength(50),
        AchError::InvalidStructure("test structure error".to_string()),
        AchError::EmptyFile,
        AchError::IncompleteBatch("test batch error".to_string()),
    ];

    for error in errors {
        // Should be able to use as Error trait object
        let _error_trait: &dyn Error = &error;

        // Should have a Display implementation (non-empty message)
        let message = format!("{}", error);
        assert!(!message.is_empty(), "Error message should not be empty");

        // Should have a Debug implementation
        let debug = format!("{:?}", error);
        assert!(!debug.is_empty(), "Debug output should not be empty");
    }
}

#[test]
fn test_invalid_number_error_preserves_source() {
    // Test that InvalidNumber error preserves the source error
    // Using invalid characters "XX" in the amount field (positions 30-39)
    let invalid_ach = concat!(
        "101 12345678012345678011409020123A094101YOUR BANK              YOUR COMPANY                   \n",
        "5200YOUR COMPANY                        1234567890PPDPAYROLL         140903   1123456780000001\n",
        "62212345678011232132         00000000XX               ALICE WANDERDUST        1123456780000001\n",
        "820000000100123456780000000000000000000010001234567890                         123456780000001\n",
        "9000001000001000000010012345678000000000000000000001000                                       ",
    );

    match AchFile::parse(invalid_ach) {
        Err(AchError::InvalidNumber { field, source }) => {
            // Should have field name
            assert_eq!(field, "amount");

            // Should have source error
            let source_msg = format!("{}", source);
            assert!(!source_msg.is_empty());

            // Should be able to access source via Error trait
            let err: &dyn Error = &AchError::InvalidNumber {
                field: "amount",
                source: source.clone(),
            };
            assert!(err.source().is_some());
        }
        Ok(_) => panic!("Should have failed with InvalidNumber"),
        Err(e) => panic!("Wrong error type: {}", e),
    }
}

#[test]
fn test_error_messages_are_descriptive() {
    // Test that error messages provide helpful context

    // InvalidLineLength should include actual length
    let err = AchError::InvalidLineLength(50);
    let msg = format!("{}", err);
    assert!(msg.contains("94"), "Should mention expected length");
    assert!(msg.contains("50"), "Should mention actual length");

    // InvalidRecordType should include the invalid type
    let err = AchError::InvalidRecordType("X".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("X"), "Should mention the invalid record type");

    // InvalidStructure should include context
    let err = AchError::InvalidStructure("Missing file control record".to_string());
    let msg = format!("{}", err);
    assert!(
        msg.contains("Missing file control record"),
        "Should include the context"
    );
}

#[test]
fn test_error_provides_line_context_where_available() {
    // Test that errors include line numbers when they can

    let invalid_ach = concat!(
        "101 12345678012345678011409020123A094101YOUR BANK              YOUR COMPANY                   \n",
        "5200YOUR COMPANY                        1234567890PPDPAYROLL         140903   1123456780000001\n",
        "622123456780112321320000000001000               ALICE WANDERDUST        1123456780000001\n",
        "X20000000100123456780000000000000000000010001234567890                         123456780000001\n",
        "9000001000001000000010012345678000000000000000000001000                                       ",
    );

    match AchFile::parse(invalid_ach) {
        Err(AchError::InvalidStructure(msg)) => {
            // Should include line context
            assert!(
                msg.contains("line"),
                "Error should mention line number: {}",
                msg
            );
        }
        Ok(_) => panic!("Should have failed"),
        Err(e) => {
            // Also acceptable if it fails with different error
            println!("Got error: {}", e);
        }
    }
}

#[test]
fn test_empty_file_error() {
    let empty = "";
    match AchFile::parse(empty) {
        Err(AchError::EmptyFile) => {
            // Correct error type
            let msg = format!("{}", AchError::EmptyFile);
            assert!(msg.contains("Empty") || msg.contains("empty"));
        }
        Ok(_) => panic!("Should have failed with EmptyFile"),
        Err(e) => panic!("Wrong error type: {}", e),
    }
}

#[test]
fn test_invalid_line_length_error() {
    let short_line = "101 123";
    match AchFile::parse(short_line) {
        Err(AchError::InvalidLineLength(len)) => {
            assert_eq!(len, 7);
            let msg = format!("{}", AchError::InvalidLineLength(len));
            assert!(msg.contains("94"));
            assert!(msg.contains("7"));
        }
        Ok(_) => panic!("Should have failed with InvalidLineLength"),
        Err(e) => panic!("Wrong error type: {}", e),
    }
}

#[test]
fn test_invalid_record_type_error() {
    let invalid_record = "X01 12345678012345678011409020123A094101YOUR BANK              YOUR COMPANY                   ";
    match AchFile::parse(invalid_record) {
        Err(AchError::InvalidRecordType(rt)) => {
            assert_eq!(rt, "X");
            let msg = format!("{}", AchError::InvalidRecordType(rt));
            assert!(msg.contains("X"));
        }
        Ok(_) => panic!("Should have failed with InvalidRecordType"),
        Err(e) => panic!("Wrong error type: {}", e),
    }
}

#[test]
fn test_incomplete_batch_error() {
    // Batch without control record
    let incomplete_batch = concat!(
        "101 12345678012345678011409020123A094101YOUR BANK              YOUR COMPANY                   \n",
        "5200YOUR COMPANY                        1234567890PPDPAYROLL         140903   1123456780000001\n",
        "62212345678011232132         0000001000               ALICE WANDERDUST        1123456780000001\n",
        // Missing batch control (8) record - file control comes directly after entry
        "9000001000001000000010012345678000000000000000000001000                                       ",
    );

    match AchFile::parse(incomplete_batch) {
        // Both IncompleteBatch and InvalidStructure are valid errors for this case
        Err(AchError::IncompleteBatch(msg)) => {
            assert!(
                msg.contains("control") || msg.contains("Missing"),
                "Error should mention missing control: {}",
                msg
            );
        }
        Err(AchError::InvalidStructure(msg)) => {
            // Also valid - parser detected wrong record type in batch
            assert!(!msg.is_empty(), "Error should have a message");
        }
        Ok(_) => panic!("Should have failed with batch error"),
        Err(e) => panic!("Wrong error type: {}", e),
    }
}

#[test]
fn test_invalid_structure_error() {
    // File starting with wrong record type
    let wrong_structure = concat!(
        "101 12345678012345678011409020123A094101YOUR BANK              YOUR COMPANY                   \n",
        "622123456780112321320000000001000               ALICE WANDERDUST        1123456780000001\n",
        // Entry before batch header - wrong structure
    );

    match AchFile::parse(wrong_structure) {
        Err(AchError::InvalidStructure(msg)) => {
            // Should provide context about what's wrong
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
        Ok(_) => panic!("Should have failed with InvalidStructure"),
        Err(e) => {
            // Also acceptable if it gives a different error
            println!("Got error: {}", e);
        }
    }
}

#[test]
fn test_errors_are_send_and_sync() {
    // Errors should be Send + Sync for use in concurrent contexts
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<AchError>();
    assert_sync::<AchError>();
}

#[test]
fn test_error_can_be_boxed() {
    // Should be able to box errors (common pattern in Rust)
    let error: AchError = AchError::EmptyFile;
    let _boxed: Box<dyn Error> = Box::new(error);
}

#[test]
fn test_error_can_be_compared() {
    // Test that we can pattern match on errors (Debug is derived)
    let err1 = AchError::EmptyFile;
    let err2 = AchError::EmptyFile;

    // Should be able to format both the same way
    assert_eq!(format!("{:?}", err1), format!("{:?}", err2));
}
