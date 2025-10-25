//! ACH record type definitions following NACHA specifications.
//!
//! Each record type represents a specific line in an ACH file.
//! All ACH records are exactly 94 characters long.

/// File Header Record (Record Type 1)
///
/// The file header record designates physical file characteristics and
/// identifies the immediate destination and origin of the entries within the file.
#[derive(Debug, Clone)]
pub struct FileHeader<'a> {
    /// Record Type Code (always "1")
    pub record_type: &'a str,

    /// Priority Code (01-99)
    pub priority_code: &'a str,

    /// Immediate Destination (10 characters) - Routing number with leading space
    pub immediate_destination: &'a str,

    /// Immediate Origin (10 characters) - Company ID with leading space
    pub immediate_origin: &'a str,

    /// File Creation Date (YYMMDD)
    pub file_creation_date: &'a str,

    /// File Creation Time (HHMM)
    pub file_creation_time: &'a str,

    /// File ID Modifier (A-Z, 0-9)
    pub file_id_modifier: &'a str,

    /// Record Size (always "094")
    pub record_size: &'a str,

    /// Blocking Factor (always "10")
    pub blocking_factor: &'a str,

    /// Format Code (always "1")
    pub format_code: &'a str,

    /// Immediate Destination Name (23 characters)
    pub immediate_destination_name: &'a str,

    /// Immediate Origin Name (23 characters)
    pub immediate_origin_name: &'a str,

    /// Reference Code (8 characters)
    pub reference_code: &'a str,
}

/// Batch Header Record (Record Type 5)
///
/// The batch header record identifies the batch and provides summary
/// information about the entries in the batch.
#[derive(Debug, Clone)]
pub struct BatchHeader<'a> {
    /// Record Type Code (always "5")
    pub record_type: &'a str,

    /// Service Class Code (200, 220, 225)
    /// - 200: Mixed debits and credits
    /// - 220: Credits only
    /// - 225: Debits only
    pub service_class_code: &'a str,

    /// Company Name (16 characters)
    pub company_name: &'a str,

    /// Company Discretionary Data (20 characters)
    pub company_discretionary_data: &'a str,

    /// Company Identification (10 characters) - Tax ID
    pub company_identification: &'a str,

    /// Standard Entry Class Code (3 characters) - PPD, CCD, WEB, etc.
    pub standard_entry_class_code: &'a str,

    /// Company Entry Description (10 characters)
    pub company_entry_description: &'a str,

    /// Company Descriptive Date (6 characters)
    pub company_descriptive_date: &'a str,

    /// Effective Entry Date (YYMMDD)
    pub effective_entry_date: &'a str,

    /// Settlement Date (Julian, 3 characters)
    pub settlement_date: &'a str,

    /// Originator Status Code (1 character)
    pub originator_status_code: &'a str,

    /// Originating DFI Identification (8 characters) - First 8 digits of routing number
    pub originating_dfi_identification: &'a str,

    /// Batch Number (7 characters)
    pub batch_number: &'a str,
}

/// Entry Detail Record (Record Type 6)
///
/// Contains the details of individual transactions within a batch.
#[derive(Debug, Clone)]
pub struct EntryDetail<'a> {
    /// Record Type Code (always "6")
    pub record_type: &'a str,

    /// Transaction Code (22, 23, 27, 28, 32, 33, 37, 38)
    pub transaction_code: &'a str,

    /// Receiving DFI Identification (8 characters) - First 8 digits of routing number
    pub receiving_dfi_identification: &'a str,

    /// Check Digit (1 character) - 9th digit of routing number
    pub check_digit: &'a str,

    /// DFI Account Number (17 characters)
    pub dfi_account_number: &'a str,

    /// Amount (10 characters) - In cents, no decimal
    pub amount: u64,

    /// Individual Identification Number (15 characters)
    pub individual_identification_number: &'a str,

    /// Individual Name (22 characters)
    pub individual_name: &'a str,

    /// Discretionary Data (2 characters)
    pub discretionary_data: &'a str,

    /// Addenda Record Indicator (0 or 1)
    pub addenda_record_indicator: &'a str,

    /// Trace Number (15 characters)
    pub trace_number: &'a str,

    /// Optional addenda records
    pub addenda: Vec<Addenda<'a>>,
}

/// Addenda Record (Record Type 7)
///
/// Provides additional information for an entry detail record.
#[derive(Debug, Clone)]
pub struct Addenda<'a> {
    /// Record Type Code (always "7")
    pub record_type: &'a str,

    /// Addenda Type Code (05 for most types)
    pub addenda_type_code: &'a str,

    /// Payment Related Information (80 characters)
    pub payment_related_information: &'a str,

    /// Addenda Sequence Number (4 characters)
    pub addenda_sequence_number: &'a str,

    /// Entry Detail Sequence Number (7 characters)
    pub entry_detail_sequence_number: &'a str,
}

/// Batch Control Record (Record Type 8)
///
/// Contains totals and counts for the entries in the batch.
#[derive(Debug, Clone)]
pub struct BatchControl {
    /// Record Type Code (always "8")
    pub record_type: String,

    /// Service Class Code (must match batch header)
    pub service_class_code: String,

    /// Entry/Addenda Count
    pub entry_addenda_count: u64,

    /// Entry Hash - Sum of receiving DFI identification numbers
    pub entry_hash: u64,

    /// Total Debit Entry Dollar Amount (in cents)
    pub total_debit_amount: u64,

    /// Total Credit Entry Dollar Amount (in cents)
    pub total_credit_amount: u64,

    /// Company Identification (must match batch header)
    pub company_identification: String,

    /// Message Authentication Code (19 characters)
    pub message_authentication_code: String,

    /// Reserved (6 characters)
    pub reserved: String,

    /// Originating DFI Identification (8 characters)
    pub originating_dfi_identification: String,

    /// Batch Number (must match batch header)
    pub batch_number: String,
}

/// File Control Record (Record Type 9)
///
/// Contains totals and counts for the entire file.
#[derive(Debug, Clone)]
pub struct FileControl {
    /// Record Type Code (always "9")
    pub record_type: String,

    /// Batch Count
    pub batch_count: u64,

    /// Block Count
    pub block_count: u64,

    /// Entry/Addenda Count
    pub entry_addenda_count: u64,

    /// Entry Hash - Sum of all entry hashes
    pub entry_hash: u64,

    /// Total Debit Entry Dollar Amount in File (in cents)
    pub total_debit_amount: u64,

    /// Total Credit Entry Dollar Amount in File (in cents)
    pub total_credit_amount: u64,

    /// Reserved (39 characters)
    pub reserved: String,
}
