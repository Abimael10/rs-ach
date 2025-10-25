# rs-ach

[![Crates.io](https://img.shields.io/crates/v/rs-ach)](https://crates.io/crates/rs-ach)
[![Documentation](https://docs.rs/rs-ach/badge.svg)](https://docs.rs/rs-ach)
[![License](https://img.shields.io/crates/l/rs-ach)](LICENSE)

ACH (Automated Clearing House) file parser for Rust following the NACHA (National Automated Clearing House Association) specifications.

## Features

- Parse ACH file headers and batch records
- Support for PPD, CCD, and other standard entry class codes
- Entry detail records with addenda support
- Type-safe parsing with comprehensive error handling
- Zero-copy parsing for maximum performance

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rs-ach = "0.1.2"
```
## Example

```rust
use rs_ach::AchFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read ACH file content
    let ach_content = std::fs::read_to_string("sample.ach")?;

    // Parse the ACH file
    let ach_file = AchFile::parse(&ach_content)?;

    // Access file header information
    println!("File Header:");
    println!("  Destination: {}", ach_file.file_header.immediate_destination.trim());
    println!("  Origin: {}", ach_file.file_header.immediate_origin.trim());
    println!("  Creation Date: {}", ach_file.file_header.file_creation_date);

    // Iterate through batches
    for (i, batch) in ach_file.batches.iter().enumerate() {
        println!("\nBatch {}:", i + 1);
        println!("  Company: {}", batch.header.company_name.trim());
        println!("  SEC Code: {}", batch.header.standard_entry_class_code);
        println!("  Description: {}", batch.header.company_entry_description.trim());

        // Iterate through entries in the batch
        for (j, entry) in batch.entries.iter().enumerate() {
            println!("    Entry {}:", j + 1);
            println!("      Transaction Code: {}", entry.transaction_code);
            println!("      Routing Number: {}{}",
                entry.receiving_dfi_identification,
                entry.check_digit
            );
            println!("      Account: {}", entry.dfi_account_number.trim());
            println!("      Amount: ${:.2}", entry.amount as f64 / 100.0);
            println!("      Name: {}", entry.individual_name.trim());

            // Check for addenda records
            if !entry.addenda.is_empty() {
                println!("      Addenda:");
                for addenda in &entry.addenda {
                    println!("        {}", addenda.payment_related_information.trim());
                }
            }
        }
    }

    // Access file control totals
    println!("\nFile Control:");
    println!("  Batch Count: {}", ach_file.file_control.batch_count);
    println!("  Total Debits: ${:.2}", ach_file.file_control.total_debit_amount as f64 / 100.0);
    println!("  Total Credits: ${:.2}", ach_file.file_control.total_credit_amount as f64 / 100.0);

    Ok(())
}
```

## ACH File Format

An ACH file consists of the following record types:

### File Header (Record Type 1)

Contains file-level information including:
- Immediate destination and origin (routing numbers)
- File creation date and time
- File identification

### Batch Header (Record Type 5)

Contains batch-level information including:
- Service class code (200=mixed, 220=credits only, 225=debits only)
- Company name and identification
- Standard entry class code (PPD, CCD, WEB, etc.)
- Effective entry date

### Entry Detail (Record Type 6)

Contains individual transaction information including:
- Transaction code (22=checking credit, 27=checking debit, 32=savings credit, 37=savings debit)
- Receiving DFI identification (routing number)
- Account number
- Amount (in cents)
- Individual name

### Addenda (Record Type 7)

Optional additional information for an entry detail record.

### Batch Control (Record Type 8)

Contains batch totals and counts:
- Entry/addenda count
- Entry hash
- Total debit and credit amounts

### File Control (Record Type 9)

Contains file-level totals and counts:
- Batch count
- Block count
- Total debit and credit amounts

## Error Handling

The parser provides detailed error information through the `AchError` enum:

```rust
use rs_ach::{AchFile, AchError};

match AchFile::parse(content) {
    Ok(ach_file) => {
        // Process the file
    },
    Err(AchError::InvalidLineLength(len)) => {
        eprintln!("Invalid line length: {}", len);
    },
    Err(AchError::InvalidRecordType(rt)) => {
        eprintln!("Invalid record type: {}", rt);
    },
    Err(e) => {
        eprintln!("Parse error: {}", e);
    }
}
```

## Testing

Run the test suite:

```bash
cargo test
```

The test suite includes:
- Basic ACH file parsing
- Credits-only batches (service class 220)
- Debits-only batches (service class 225)
- Entries with addenda records
- Error handling for invalid formats

## License

MIT
