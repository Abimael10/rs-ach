//! ACH file parser implementation.

use crate::error::AchError;
use crate::records::*;
use crate::{AchFile, Batch};

/// Parse a complete ACH file from a string.
///
/// # Arguments
///
/// * `content` - The complete ACH file content
///
/// # Returns
///
/// Returns a parsed `AchFile` or an error if parsing fails.
pub fn parse_ach_file<'a>(content: &'a str) -> Result<AchFile<'a>, AchError> {
    let lines: Vec<&'a str> = content
        .lines()
        .filter(|line| !line.chars().all(|c| c == '9'))
        .collect();

    if lines.is_empty() {
        return Err(AchError::EmptyFile);
    }

    let mut line_idx = 0;

    // Parse file header (must be first)
    let file_header = parse_file_header(lines[line_idx])?;
    line_idx += 1;

    // Parse batches
    let mut batches = Vec::new();
    while line_idx < lines.len() {
        let record_type = get_record_type(lines[line_idx])?;

        if record_type == "5" {
            let batch = parse_batch(&lines, &mut line_idx)?;
            batches.push(batch);
        } else if record_type == "9" {
            break;
        } else {
            return Err(AchError::InvalidStructure(format!(
                "Unexpected record type '{record_type}' at line {line_idx}"
            )));
        }
    }

    // Parse file control (must be last)
    if line_idx >= lines.len() {
        return Err(AchError::InvalidStructure(
            "Missing file control record".to_string(),
        ));
    }

    let file_control = parse_file_control(lines[line_idx])?;

    Ok(AchFile {
        file_header,
        batches,
        file_control,
    })
}

/// Parse a single batch including header, entries, and control.
fn parse_batch<'a>(lines: &[&'a str], line_idx: &mut usize) -> Result<Batch<'a>, AchError> {
    // Parse batch header
    let header = parse_batch_header(lines[*line_idx])?;
    *line_idx += 1;

    // Parse entries
    let mut entries = Vec::new();
    while *line_idx < lines.len() {
        let record_type = get_record_type(lines[*line_idx])?;

        if record_type == "6" {
            let mut entry = parse_entry_detail(lines[*line_idx])?;
            *line_idx += 1;

            // Check for addenda records
            while *line_idx < lines.len() {
                let next_record_type = get_record_type(lines[*line_idx])?;
                if next_record_type == "7" {
                    let addenda = parse_addenda(lines[*line_idx])?;
                    entry.addenda.push(addenda);
                    *line_idx += 1;
                } else {
                    break;
                }
            }

            entries.push(entry);
        } else if record_type == "8" {
            break;
        } else {
            return Err(AchError::InvalidStructure(format!(
                "Unexpected record type '{record_type}' in batch at line {line_idx}"
            )));
        }
    }

    // Parse batch control
    if *line_idx >= lines.len() {
        return Err(AchError::IncompleteBatch(
            "Missing batch control record".to_string(),
        ));
    }

    let control = parse_batch_control(lines[*line_idx])?;
    *line_idx += 1;

    Ok(Batch {
        header,
        entries,
        control,
    })
}

/// Get the record type (first character) from a line.
fn get_record_type(line: &str) -> Result<&str, AchError> {
    if line.is_empty() {
        return Err(AchError::InvalidLineLength(0));
    }
    Ok(&line[0..1])
}

/// Validate that a line is exactly 94 characters.
fn validate_line_length(line: &str) -> Result<(), AchError> {
    if line.len() != 94 {
        return Err(AchError::InvalidLineLength(line.len()));
    }
    Ok(())
}

/// Parse a file header record (type 1).
fn parse_file_header(line: &str) -> Result<FileHeader, AchError> {
    validate_line_length(line)?;

    let record_type = &line[0..1];
    if record_type != "1" {
        return Err(AchError::InvalidRecordType(record_type.to_string()));
    }

    Ok(FileHeader {
        record_type,
        priority_code: &line[1..3],
        immediate_destination: &line[3..13],
        immediate_origin: &line[13..23],
        file_creation_date: &line[23..29],
        file_creation_time: &line[29..33],
        file_id_modifier: &line[33..34],
        record_size: &line[34..37],
        blocking_factor: &line[37..39],
        format_code: &line[39..40],
        immediate_destination_name: &line[40..63],
        immediate_origin_name: &line[63..86],
        reference_code: &line[86..94],
    })
}

/// Parse a batch header record (type 5).
fn parse_batch_header(line: &str) -> Result<BatchHeader, AchError> {
    validate_line_length(line)?;

    let record_type = &line[0..1];
    if record_type != "5" {
        return Err(AchError::InvalidRecordType(record_type.to_string()));
    }

    Ok(BatchHeader {
        record_type,
        service_class_code: &line[1..4],
        company_name: &line[4..20],
        company_discretionary_data: &line[20..40],
        company_identification: &line[40..50],
        standard_entry_class_code: &line[50..53],
        company_entry_description: &line[53..63],
        company_descriptive_date: &line[63..69],
        effective_entry_date: &line[69..75],
        settlement_date: &line[75..78],
        originator_status_code: &line[78..79],
        originating_dfi_identification: &line[79..87],
        batch_number: &line[87..94],
    })
}

/// Parse an entry detail record (type 6).
fn parse_entry_detail(line: &str) -> Result<EntryDetail, AchError> {
    validate_line_length(line)?;

    let record_type = &line[0..1];
    if record_type != "6" {
        return Err(AchError::InvalidRecordType(record_type.to_string()));
    }

    // Parse amount field (positions 29-39, 10 characters)
    let amount_str = line[29..39].trim();
    let amount = amount_str
        .parse::<u64>()
        .map_err(|e| AchError::InvalidNumber {
            field: "amount",
            source: e,
        })?;

    Ok(EntryDetail {
        record_type,
        transaction_code: &line[1..3],
        receiving_dfi_identification: &line[3..11],
        check_digit: &line[11..12],
        dfi_account_number: &line[12..29],
        amount,
        individual_identification_number: &line[39..54],
        individual_name: &line[54..76],
        discretionary_data: &line[76..78],
        addenda_record_indicator: &line[78..79],
        trace_number: &line[79..94],
        addenda: Vec::new(),
    })
}

/// Parse an addenda record (type 7).
fn parse_addenda(line: &str) -> Result<Addenda, AchError> {
    validate_line_length(line)?;

    let record_type = &line[0..1];
    if record_type != "7" {
        return Err(AchError::InvalidRecordType(record_type.to_string()));
    }

    Ok(Addenda {
        record_type,
        addenda_type_code: &line[1..3],
        payment_related_information: &line[3..83],
        addenda_sequence_number: &line[83..87],
        entry_detail_sequence_number: &line[87..94],
    })
}

/// Parse a batch control record (type 8).
fn parse_batch_control(line: &str) -> Result<BatchControl, AchError> {
    validate_line_length(line)?;

    let record_type = &line[0..1];
    if record_type != "8" {
        return Err(AchError::InvalidRecordType(record_type.to_string()));
    }

    Ok(BatchControl {
        record_type: line[0..1].to_string(),
        service_class_code: line[1..4].to_string(),
        entry_addenda_count: parse_u64(&line[4..10], "entry_addenda_count")?,
        entry_hash: parse_u64(&line[10..20], "entry_hash")?,
        total_debit_amount: parse_u64(&line[20..32], "total_debit_amount")?,
        total_credit_amount: parse_u64(&line[32..44], "total_credit_amount")?,
        company_identification: line[44..54].to_string(),
        message_authentication_code: line[54..73].to_string(),
        reserved: line[73..79].to_string(),
        originating_dfi_identification: line[79..87].to_string(),
        batch_number: line[87..94].to_string(),
    })
}

/// Parse a file control record (type 9).
fn parse_file_control(line: &str) -> Result<FileControl, AchError> {
    validate_line_length(line)?;

    let record_type = &line[0..1];
    if record_type != "9" {
        return Err(AchError::InvalidRecordType(record_type.to_string()));
    }

    Ok(FileControl {
        record_type: line[0..1].to_string(),
        batch_count: parse_u64(&line[1..7], "batch_count")?,
        block_count: parse_u64(&line[7..13], "block_count")?,
        entry_addenda_count: parse_u64(&line[13..21], "entry_addenda_count")?,
        entry_hash: parse_u64(&line[21..31], "entry_hash")?,
        total_debit_amount: parse_u64(&line[31..43], "total_debit_amount")?,
        total_credit_amount: parse_u64(&line[43..55], "total_credit_amount")?,
        reserved: line[55..94].to_string(),
    })
}

/// Helper function to parse a u64 from a string slice.
fn parse_u64(s: &str, field_name: &'static str) -> Result<u64, AchError> {
    s.trim()
        .parse::<u64>()
        .map_err(|e| AchError::InvalidNumber {
            field: field_name,
            source: e,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u64_valid() {
        assert_eq!(parse_u64("12345", "test").unwrap(), 12345);
        assert_eq!(parse_u64("  12345  ", "test").unwrap(), 12345);
        assert_eq!(parse_u64("0000012345", "test").unwrap(), 12345);
    }

    #[test]
    fn test_parse_u64_invalid() {
        assert!(parse_u64("abc", "test").is_err());
        assert!(parse_u64("12.34", "test").is_err());
    }

    #[test]
    fn test_get_record_type() {
        assert_eq!(get_record_type("101").unwrap(), "1");
        assert_eq!(get_record_type("5200").unwrap(), "5");
        assert_eq!(get_record_type("6221").unwrap(), "6");
        assert!(get_record_type("").is_err());
    }

    #[test]
    fn test_validate_line_length() {
        let valid = "1".repeat(94);
        assert!(validate_line_length(&valid).is_ok());

        let too_short = "1".repeat(93);
        assert!(validate_line_length(&too_short).is_err());

        let too_long = "1".repeat(95);
        assert!(validate_line_length(&too_long).is_err());
    }

    #[test]
    fn test_parse_file_header() {
        let header = "101 12345678012345678011409020123A094101YOUR BANK              YOUR COMPANY                   ";
        let result = parse_file_header(header);
        assert!(result.is_ok());

        let fh = result.unwrap();
        assert_eq!(fh.record_type, "1");
        assert_eq!(fh.priority_code, "01");
        assert_eq!(fh.immediate_destination.trim(), "123456780");
        assert_eq!(fh.immediate_origin.trim(), "1234567801");
        assert_eq!(fh.file_creation_date, "140902");
        assert_eq!(fh.file_creation_time, "0123");
        assert_eq!(fh.file_id_modifier, "A");
    }

    #[test]
    fn test_parse_batch_header() {
        let header = "5200YOUR COMPANY                        1234567890PPDPAYROLL         140903   1123456780000001";
        let result = parse_batch_header(header);
        assert!(result.is_ok());

        let bh = result.unwrap();
        assert_eq!(bh.record_type, "5");
        assert_eq!(bh.service_class_code, "200");
        assert_eq!(bh.company_name.trim(), "YOUR COMPANY");
        assert_eq!(bh.company_identification, "1234567890");
        assert_eq!(bh.standard_entry_class_code, "PPD");
        assert_eq!(bh.company_entry_description.trim(), "PAYROLL");
    }

    #[test]
    fn test_parse_entry_detail() {
        let entry = "62212345678011232132         0000001000               ALICE WANDERDUST        1123456780000001";
        let result = parse_entry_detail(entry);
        assert!(result.is_ok());

        let ed = result.unwrap();
        assert_eq!(ed.record_type, "6");
        assert_eq!(ed.transaction_code, "22");
        assert_eq!(ed.receiving_dfi_identification, "12345678");
        assert_eq!(ed.check_digit, "0");
        assert_eq!(ed.dfi_account_number.trim(), "11232132");
        assert_eq!(ed.amount, 1000);
        assert_eq!(ed.individual_name.trim(), "ALICE WANDERDUST");
    }

    #[test]
    fn test_parse_addenda() {
        let addenda = "705HERE IS SOME ADDITIONAL INFORMATION                                             00000000001";
        let result = parse_addenda(addenda);
        assert!(result.is_ok());

        let add = result.unwrap();
        assert_eq!(add.record_type, "7");
        assert_eq!(add.addenda_type_code, "05");
        assert!(
            add.payment_related_information
                .starts_with("HERE IS SOME ADDITIONAL")
        );
    }

    #[test]
    fn test_parse_batch_control() {
        let control = "820000000400370145870000000150000000000022131234567890                         123456780000001";
        let result = parse_batch_control(control);
        assert!(result.is_ok());

        let bc = result.unwrap();
        assert_eq!(bc.record_type, "8");
        assert_eq!(bc.service_class_code, "200");
        assert_eq!(bc.entry_addenda_count, 4);
    }

    #[test]
    fn test_parse_file_control() {
        let control = "9000001000001000000040037014587000000015000000000002213                                       ";
        let result = parse_file_control(control);
        assert!(result.is_ok());

        let fc = result.unwrap();
        assert_eq!(fc.record_type, "9");
        assert_eq!(fc.batch_count, 1);
        assert_eq!(fc.block_count, 1);
    }
}
