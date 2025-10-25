//! Integration tests for rs-ach parsing

use rs_ach::{AchError, AchFile};

/// Sample ACH file from the NACHA specification (similar to python-ach example)
/// Each line is exactly 94 characters as per NACHA standard
const SAMPLE_ACH_FILE: &str = concat!(
    "101 12345678012345678011409020123A094101YOUR BANK              YOUR COMPANY                   \n",
    "5200YOUR COMPANY                        1234567890PPDPAYROLL         140903   1123456780000001\n",
    "62212345678011232132         0000001000               ALICE WANDERDUST        1123456780000001\n",
    "705HERE IS SOME ADDITIONAL INFORMATION                                             00000000001\n",
    "627123456780234234234        0000015000               BILLY HOLIDAY           0123456780000002\n",
    "622123232318123123123        0000001213               RACHEL WELCH            0123456780000003\n",
    "820000000400370145870000000150000000000022131234567890                         123456780000001\n",
    "9000001000001000000040037014587000000015000000000002213                                       ",
);

#[test]
fn test_parse_basic_ach_file() {
    let result = AchFile::parse(SAMPLE_ACH_FILE);
    assert!(
        result.is_ok(),
        "Failed to parse ACH file: {:?}",
        result.err()
    );

    let ach_file = result.unwrap();

    // Verify file header
    assert_eq!(ach_file.file_header.record_type, "1");
    assert_eq!(ach_file.file_header.priority_code, "01");
    assert_eq!(
        ach_file.file_header.immediate_destination.trim(),
        "123456780"
    );
    assert_eq!(ach_file.file_header.immediate_origin.trim(), "1234567801");

    // Verify we have one batch
    assert_eq!(ach_file.batches.len(), 1);

    let batch = &ach_file.batches[0];

    // Verify batch header
    assert_eq!(batch.header.record_type, "5");
    assert_eq!(batch.header.service_class_code, "200");
    assert_eq!(batch.header.company_name.trim(), "YOUR COMPANY");
    assert_eq!(batch.header.standard_entry_class_code, "PPD");
    assert_eq!(batch.header.company_entry_description.trim(), "PAYROLL");

    // Verify entries
    assert_eq!(batch.entries.len(), 3);

    // First entry (with addenda)
    let entry1 = &batch.entries[0];
    assert_eq!(entry1.transaction_code, "22");
    assert_eq!(entry1.receiving_dfi_identification, "12345678");
    assert_eq!(entry1.dfi_account_number.trim(), "11232132");
    assert_eq!(entry1.amount, 1000);
    assert_eq!(entry1.individual_name.trim(), "ALICE WANDERDUST");
    assert_eq!(entry1.addenda.len(), 1);
    assert_eq!(
        entry1.addenda[0]
            .payment_related_information
            .trim()
            .starts_with("HERE IS SOME ADDITIONAL"),
        true
    );

    // Second entry (no addenda)
    let entry2 = &batch.entries[1];
    assert_eq!(entry2.transaction_code, "27");
    assert_eq!(entry2.amount, 15000);
    assert_eq!(entry2.individual_name.trim(), "BILLY HOLIDAY");
    assert_eq!(entry2.addenda.len(), 0);

    // Third entry
    let entry3 = &batch.entries[2];
    assert_eq!(entry3.transaction_code, "22");
    assert_eq!(entry3.amount, 1213);
    assert_eq!(entry3.individual_name.trim(), "RACHEL WELCH");

    // Verify batch control
    assert_eq!(batch.control.record_type, "8");
    assert_eq!(batch.control.service_class_code, "200");
    assert_eq!(batch.control.entry_addenda_count, 4);

    // Verify file control
    assert_eq!(ach_file.file_control.record_type, "9");
    assert_eq!(ach_file.file_control.batch_count, 1);
}

#[test]
fn test_empty_file() {
    let result = AchFile::parse("");
    assert!(matches!(result, Err(AchError::EmptyFile)));
}

#[test]
fn test_invalid_line_length() {
    let invalid_ach = "101 123";
    let result = AchFile::parse(invalid_ach);
    assert!(matches!(result, Err(AchError::InvalidLineLength(_))));
}

#[test]
fn test_invalid_record_type() {
    let invalid_ach = concat!(
        "X01 123456780 1234567801409020123A094101YOUR BANK              YOUR COMPANY                   ",
    );
    let result = AchFile::parse(invalid_ach);
    assert!(matches!(result, Err(AchError::InvalidRecordType(_))));
}

#[test]
fn test_credits_only_batch() {
    let credits_ach = concat!(
        "101 12345678012345678011409020123A094101YOUR BANK              YOUR COMPANY                   \n",
        "5220YOUR COMPANY                        1234567890PPDPAYROLL         140903   1123456780000001\n",
        "62212345678011232132         0000001000               ALICE WANDERDUST        0123456780000001\n",
        "622123232318123123123        0000001213               RACHEL WELCH            0123456780000002\n",
        "820000000200246913980000000000000000000022131234567890                         123456780000001\n",
        "9000001000001000000020024691398000000000000000000002213                                       ",
    );

    let result = AchFile::parse(credits_ach);
    assert!(result.is_ok());

    let ach_file = result.unwrap();
    let batch = &ach_file.batches[0];

    // Service class 220 = credits only
    assert_eq!(batch.header.service_class_code, "220");
    assert_eq!(batch.entries.len(), 2);

    // Both entries should be credits (transaction code 22)
    assert_eq!(batch.entries[0].transaction_code, "22");
    assert_eq!(batch.entries[1].transaction_code, "22");
}

#[test]
fn test_debits_only_batch() {
    let debits_ach = concat!(
        "101 12345678012345678011409020123A094101YOUR BANK              YOUR COMPANY                   \n",
        "5225YOUR COMPANY                        1234567890PPDPAYROLL         140903   1123456780000001\n",
        "627123456780234234234        0000015000               BILLY HOLIDAY           0123456780000001\n",
        "627123456780999999999        0000005000               JANE DOE                0123456780000002\n",
        "820000000200370145870000000200000000000000001234567890                         123456780000001\n",
        "9000001000001000000020037014587000000020000000000000000                                       ",
    );

    let result = AchFile::parse(debits_ach);
    assert!(result.is_ok());

    let ach_file = result.unwrap();
    let batch = &ach_file.batches[0];

    // Service class 225 = debits only
    assert_eq!(batch.header.service_class_code, "225");
    assert_eq!(batch.entries.len(), 2);

    // Both entries should be debits (transaction code 27)
    assert_eq!(batch.entries[0].transaction_code, "27");
    assert_eq!(batch.entries[1].transaction_code, "27");
}
