//! # iXBRL Transformation Registry
//!
//! Handles the normalization of iXBRL fact values based on the `format` attribute.
//! This implements various transformation registries, including the SEC-specific one.
//!
//! ## Supported Transformations
//!
//! ### SEC-Specific (ixt-sec namespace)
//! - `boolballotbox`: Converts checkbox characters (☐☑☒) to boolean strings
//! - `numwordsen`: Converts English number words to numeric strings
//! - `durwordsen`: Converts English duration words to ISO 8601 duration format
//! - `durday`: Converts day counts to ISO 8601 duration (e.g., "30" → "P30D")
//! - `durmonth`: Converts month counts to ISO 8601 duration (e.g., "12" → "P12M")
//! - `exchnameen`: Normalizes exchange names to standard codes (NYSE, NASDAQ)
//! - `stateprovnameen`: Converts US state/province names to 2-letter codes
//! - `entityfilercategoryen`: Normalizes entity filer categories
//! - `edgarprovcountryen`: Normalizes EDGAR province/country names
//!
//! ### Standard (ixt namespace)
//! - `num-dot-decimal` / `numdotdecimal`: Removes comma thousands separators from numbers
//! - `zerodash` / `zero-dash`: Converts dash character to zero for numeric fields  
//! - `fixed-true/false/zero`: Returns fixed boolean or numeric values
//! - `booleanfalse/true`: Returns boolean strings
//! - `date-monthname-day-year-en` / `datemonthdayyearen`: Converts "Month Day, Year" to ISO 8601
//!
//! See: https://www.xbrl.org/specification/inlinexbrl-transformation-rules-registry-4/

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

// A transformation is a function that takes a string slice and returns a normalized String.
type Transformation = fn(&str) -> Result<String, TransformationError>;

// The registry is a map from a format name (e.g., "numwordsen") to a transformation function.
pub type TransformationRegistry = HashMap<&'static str, Transformation>;

#[derive(Debug, thiserror::Error)]
pub enum TransformationError {
    #[error("Transformation '{0}' not implemented for registry '{1}'")]
    NotImplemented(String, String),

    #[error("Invalid input value '{0}' for transformation '{1}'")]
    InvalidInput(String, String),
}

// --- Transformation Function Implementations ---

/// Converts SEC checkbox characters to boolean strings
///
/// ## Examples
/// ```
/// use xbrl::transformations::bool_ballot_box;
///
/// assert_eq!(bool_ballot_box("☐").unwrap(), "false");
/// assert_eq!(bool_ballot_box("☑").unwrap(), "true");
/// ```
pub fn bool_ballot_box(value: &str) -> Result<String, TransformationError> {
    match value.trim() {
        "☐" => Ok("false".to_string()),
        "☑" | "☒" => Ok("true".to_string()),
        _ => Err(TransformationError::InvalidInput(
            value.to_string(),
            "bool_ballot_box: expected ☐, ☑, or ☒".to_string(),
        )),
    }
}

/// Converts English number words to numeric strings
///
/// Supports numbers from zero to 999,999,999,999 (999 billion).
/// Handles both short form and variations with "and", commas, and hyphens.
///
/// ## Examples
/// - "one" → "1"
/// - "seventy thousand and one" → "70001"
/// - "nineteen hundred forty-four" → "1944"
/// - "six million four hundred thousand five" → "6400005"
pub fn num_words_en(value: &str) -> Result<String, TransformationError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("none")
        || trimmed.eq_ignore_ascii_case("no")
        || trimmed.eq_ignore_ascii_case("nil")
    {
        return Ok("0".to_string());
    }

    // Word to number mappings
    let small_numbers: HashMap<&str, i64> = [
        ("zero", 0),
        ("one", 1),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
        ("nine", 9),
        ("ten", 10),
        ("eleven", 11),
        ("twelve", 12),
        ("thirteen", 13),
        ("fourteen", 14),
        ("fifteen", 15),
        ("sixteen", 16),
        ("seventeen", 17),
        ("eighteen", 18),
        ("nineteen", 19),
        ("twenty", 20),
        ("thirty", 30),
        ("forty", 40),
        ("fifty", 50),
        ("sixty", 60),
        ("seventy", 70),
        ("eighty", 80),
        ("ninety", 90),
    ]
    .iter()
    .cloned()
    .collect();

    let magnitudes: HashMap<&str, i64> = [
        ("hundred", 100),
        ("thousand", 1_000),
        ("million", 1_000_000),
        ("billion", 1_000_000_000),
        ("trillion", 1_000_000_000_000),
    ]
    .iter()
    .cloned()
    .collect();

    // Convert to lowercase and split on whitespace, hyphens, commas, and "and"
    let lowercase = trimmed.to_lowercase();
    let words: Vec<&str> = lowercase
        .split(|c: char| c.is_whitespace() || c == '-' || c == ',')
        .filter(|s| !s.is_empty() && *s != "and")
        .collect();

    let mut result: i64 = 0;
    let mut current: i64 = 0;

    for word in words {
        if let Some(&num) = small_numbers.get(word) {
            current += num;
        } else if let Some(&mag) = magnitudes.get(word) {
            if word == "hundred" {
                current *= mag;
            } else {
                // thousand, million, billion, etc.
                current *= mag;
                result += current;
                current = 0;
            }
        } else {
            return Err(TransformationError::InvalidInput(
                value.to_string(),
                "numwordsen".to_string(),
            ));
        }
    }

    result += current;
    Ok(result.to_string())
}

/// Converts English duration words to ISO 8601 duration format
///
/// Parses durations with years, months, and/or days expressed in words or numbers.
///
/// ## Examples
/// - "9 years, 2 months" → "P9Y2M"
/// - "Five years, two months" → "P5Y2M"
/// - "three years four months no days" → "P3Y4M"
pub fn dur_words_en(value: &str) -> Result<String, TransformationError> {
    // Unicode dashes: Armenian hyphen (U+058A), Hebrew maqaf (U+05BE),
    // hyphens/dashes (U+2010-2015), small em dash (U+FE58),
    // small hyphen-minus (U+FE63), fullwidth hyphen-minus (U+FF0D), and ASCII hyphen
    // Pattern matches: "N year(s)", "M month(s)", "D day(s)" in any combination
    static DURATION_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)(?:(\d+|[a-z]+)[\s\u{058A}\u{05BE}\u{2010}-\u{2015}\u{FE58}\u{FE63}\u{FF0D}\-]*years?)?[,\s\u{058A}\u{05BE}\u{2010}-\u{2015}\u{FE58}\u{FE63}\u{FF0D}\-]*(and\s+)?(?:(\d+|[a-z]+)[\s\u{058A}\u{05BE}\u{2010}-\u{2015}\u{FE58}\u{FE63}\u{FF0D}\-]*months?)?[,\s\u{058A}\u{05BE}\u{2010}-\u{2015}\u{FE58}\u{FE63}\u{FF0D}\-]*(and\s+)?(?:(\d+|[a-z]+)[\s\u{058A}\u{05BE}\u{2010}-\u{2015}\u{FE58}\u{FE63}\u{FF0D}\-]*days?)?").unwrap()
    });

    let input = value.trim().to_lowercase();

    if let Some(caps) = DURATION_REGEX.captures(&input) {
        let mut parts = Vec::new();

        // Parse years (group 1)
        if let Some(year_str) = caps.get(1) {
            let year_val = parse_word_or_num(year_str.as_str())?;
            if year_val > 0 {
                parts.push(format!("{}Y", year_val));
            }
        }

        // Parse months (group 3, because group 2 is the "and" after years)
        if let Some(month_str) = caps.get(3) {
            let month_val = parse_word_or_num(month_str.as_str())?;
            if month_val > 0 {
                parts.push(format!("{}M", month_val));
            }
        }

        // Parse days (group 5, because group 4 is the "and" after months)
        if let Some(day_str) = caps.get(5) {
            let day_val = parse_word_or_num(day_str.as_str())?;
            if day_val > 0 {
                parts.push(format!("{}D", day_val));
            }
        }

        if parts.is_empty() {
            Ok("P0D".to_string())
        } else {
            Ok(format!("P{}", parts.join("")))
        }
    } else {
        Err(TransformationError::InvalidInput(
            value.to_string(),
            "durwordsen".to_string(),
        ))
    }
}

/// Helper function to parse a word or number string
fn parse_word_or_num(s: &str) -> Result<i64, TransformationError> {
    let trimmed = s.trim().to_lowercase();

    // Try parsing as a number first
    if let Ok(num) = trimmed.parse::<i64>() {
        return Ok(num);
    }

    // Handle special cases (matching Python's durwordZeroNoPattern and numwordsNoPattern)
    if trimmed == "no" || trimmed == "zero" || trimmed == "none" || trimmed == "nil" {
        return Ok(0);
    }

    // Try word-to-number conversion for small numbers
    let small_numbers: HashMap<&str, i64> = [
        ("one", 1),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
        ("nine", 9),
        ("ten", 10),
        ("eleven", 11),
        ("twelve", 12),
        ("thirteen", 13),
        ("fourteen", 14),
        ("fifteen", 15),
        ("sixteen", 16),
        ("seventeen", 17),
        ("eighteen", 18),
        ("nineteen", 19),
        ("twenty", 20),
        ("thirty", 30),
        ("forty", 40),
        ("fifty", 50),
        ("sixty", 60),
        ("seventy", 70),
        ("eighty", 80),
        ("ninety", 90),
    ]
    .iter()
    .cloned()
    .collect();

    if let Some(&num) = small_numbers.get(trimmed.as_str()) {
        Ok(num)
    } else {
        Err(TransformationError::InvalidInput(
            s.to_string(),
            "word_or_num".to_string(),
        ))
    }
}

/// Removes comma thousands separators from numbers
///
/// ## Examples
/// - "1,000,000" → "1000000"
/// - "123,456.789" → "123456.789"
pub fn num_dot_decimal(value: &str) -> Result<String, TransformationError> {
    Ok(value.replace(',', ""))
}

/// Returns the fixed value "false" regardless of input
pub fn fixed_false(_value: &str) -> Result<String, TransformationError> {
    Ok("false".to_string())
}

/// Returns the fixed value "true" regardless of input
pub fn fixed_true(_value: &str) -> Result<String, TransformationError> {
    Ok("true".to_string())
}

/// Returns the fixed value "0" regardless of input
pub fn fixed_zero(_value: &str) -> Result<String, TransformationError> {
    Ok("0".to_string())
}

/// Returns "false" (boolean false)
pub fn boolean_false(_value: &str) -> Result<String, TransformationError> {
    Ok("false".to_string())
}

/// Returns "true" (boolean true)
pub fn boolean_true(_value: &str) -> Result<String, TransformationError> {
    Ok("true".to_string())
}

/// Converts dash (-) to zero (0) for numeric fields
///
/// This is a standard iXBRL transformation used when a dash represents
/// a zero value in financial statements.
///
/// ## Examples
/// - "-" → "0"
/// - "  -  " → "0" (with whitespace)
pub fn zero_dash(value: &str) -> Result<String, TransformationError> {
    let trimmed = value.trim();
    if trimmed == "-" || trimmed == "—" || trimmed == "–" {
        Ok("0".to_string())
    } else {
        // If it's not a dash, return the original value trimmed
        Ok(trimmed.to_string())
    }
}

/// Returns positive infinity for float/double types
pub fn num_inf(_value: &str) -> Result<String, TransformationError> {
    Ok("INF".to_string())
}

/// Returns negative infinity for float/double types
pub fn num_neg_inf(_value: &str) -> Result<String, TransformationError> {
    Ok("-INF".to_string())
}

/// Returns NaN (not-a-number) for float/double types
pub fn num_nan(_value: &str) -> Result<String, TransformationError> {
    Ok("NaN".to_string())
}

/// Converts ballot box unicode characters to Yes/No text values
///
/// Translates:
/// - ☐ (U+2610 BALLOT BOX) → "No"
/// - ☑ (U+2611 BALLOT BOX WITH CHECK) → "Yes"
/// - ☒ (U+2612 BALLOT BOX WITH X) → "Yes"
pub fn yesno_ballot_box(value: &str) -> Result<String, TransformationError> {
    let input = value.trim();

    if input.is_empty() {
        return Err(TransformationError::InvalidInput(
            value.to_string(),
            "yesno_ballot_box: empty input".to_string(),
        ));
    }

    match input.chars().next() {
        Some('\u{2610}') => Ok("No".to_string()),
        Some('\u{2611}') => Ok("Yes".to_string()),
        Some('\u{2612}') => Ok("Yes".to_string()),
        _ => Err(TransformationError::InvalidInput(
            value.to_string(),
            format!("yesno_ballot_box: invalid ballot box character: {}", input),
        )),
    }
}

/// Normalizes exchange names to standard codes using SEC regex patterns
///
/// Based on SEC EDGAR specifications. Handles variations with "The", "Inc.", "LLC", etc.
///
/// ## Examples
/// - "New York Stock Exchange" → "NYSE"
/// - "The NASDAQ Stock Market, LLC" → "NASDAQ"
/// - "Cboe BZX Exchange, Inc." → "CboeBZX"
pub fn exchnameen(value: &str) -> Result<String, TransformationError> {
    // Use simple normalization for matching - remove "The", case-insensitive, trim punctuation
    let normalized = value
        .trim()
        .to_lowercase()
        .replace("the ", "")
        .replace(",", "")
        .replace(".", "")
        .replace("inc", "")
        .replace("llc", "")
        .trim()
        .to_string();

    // Match against normalized patterns
    if normalized.contains("box exchange") {
        return Ok("BOX".to_string());
    }
    if normalized.contains("cboe byx exchange") {
        return Ok("CboeBYX".to_string());
    }
    if normalized.contains("cboe bzx exchange") {
        return Ok("CboeBZX".to_string());
    }
    if normalized.contains("cboe c2 exchange") {
        return Ok("C2".to_string());
    }
    if normalized.contains("cboe edga exchange") {
        return Ok("CboeEDGA".to_string());
    }
    if normalized.contains("cboe edgx exchange") {
        return Ok("CboeEDGX".to_string());
    }
    if normalized.contains("cboe exchange") {
        return Ok("CBOE".to_string());
    }
    if normalized.contains("chicago stock exchange") {
        return Ok("CHX".to_string());
    }
    if normalized.contains("investors exchange") {
        return Ok("IEX".to_string());
    }
    if normalized.contains("miami international securities exchange") {
        return Ok("MIAX".to_string());
    }
    if normalized.contains("miax pearl") {
        return Ok("PEARL".to_string());
    }
    if normalized.contains("nasdaq bx") {
        return Ok("BX".to_string());
    }
    if normalized.contains("nasdaq gemx") {
        return Ok("GEMX".to_string());
    }
    if normalized.contains("nasdaq ise") {
        return Ok("ISE".to_string());
    }
    if normalized.contains("nasdaq mrx") {
        return Ok("MRX".to_string());
    }
    if normalized.contains("nasdaq phlx") {
        return Ok("Phlx".to_string());
    }
    // Check NYSE variants - most specific first
    if normalized.contains("nyse american") {
        return Ok("NYSEAMER".to_string());
    }
    if normalized.contains("nyse arca") {
        return Ok("NYSEArca".to_string());
    }
    if normalized.contains("nyse national") {
        return Ok("NYSENAT".to_string());
    }
    if normalized.contains("new york stock exchange") || normalized == "nyse" {
        return Ok("NYSE".to_string());
    }
    // Check NASDAQ variants
    if normalized.contains("nasdaq") {
        return Ok("NASDAQ".to_string());
    }

    Err(TransformationError::InvalidInput(
        value.to_string(),
        "exchnameen: no matching exchange pattern".to_string(),
    ))
}

/// Converts date in "Month Day, Year" format to ISO 8601 (YYYY-MM-DD)
///
/// ## Examples
/// - "August 22, 2025" → "2025-08-22"
/// - "November 29, 2021" → "2021-11-29"
pub fn date_month_day_year_en(value: &str) -> Result<String, TransformationError> {
    static DATE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)^(january|february|march|april|may|june|july|august|september|october|november|december)\s+(\d{1,2}),?\s+(\d{4})$").unwrap()
    });

    let input = value.trim();

    if let Some(caps) = DATE_REGEX.captures(input) {
        let month_str = caps.get(1).unwrap().as_str().to_lowercase();
        let day: u32 = caps.get(2).unwrap().as_str().parse().map_err(|_| {
            TransformationError::InvalidInput(
                value.to_string(),
                "date_month_day_year_en".to_string(),
            )
        })?;
        let year: u32 = caps.get(3).unwrap().as_str().parse().map_err(|_| {
            TransformationError::InvalidInput(
                value.to_string(),
                "date_month_day_year_en".to_string(),
            )
        })?;

        let month = match month_str.as_str() {
            "january" => 1,
            "february" => 2,
            "march" => 3,
            "april" => 4,
            "may" => 5,
            "june" => 6,
            "july" => 7,
            "august" => 8,
            "september" => 9,
            "october" => 10,
            "november" => 11,
            "december" => 12,
            _ => {
                return Err(TransformationError::InvalidInput(
                    value.to_string(),
                    "date_month_day_year_en".to_string(),
                ));
            }
        };

        Ok(format!("{:04}-{:02}-{:02}", year, month, day))
    } else {
        Err(TransformationError::InvalidInput(
            value.to_string(),
            "date_month_day_year_en".to_string(),
        ))
    }
}

/// Converts quarter notation to end-of-quarter date
///
/// Parses expressions like:
/// - "Q3 2023", "3rd 2023", "third quarter 2023" → "2023-09-30"
/// - "2023 Q2", "2023 second", "2023 2nd" → "2023-06-30"
/// - "last 2023" → "2023-12-31"
///
/// Based on SEC EDGAR pattern matching with two formats:
/// 1. Optional prefix + quarter + year
/// 2. Year + quarter + optional suffix
pub fn date_quarter_end(value: &str) -> Result<String, TransformationError> {
    static QUARTER_PATTERN: Lazy<Regex> = Lazy::new(|| {
        // Simplified pattern - matches common quarter notations
        // Pattern 1: Q1 2023, 1st 2023, first 2023, etc.
        // Pattern 2: 2023 Q1, 2023 first, etc.
        Regex::new(r"(?ix)^[ \t\n\r]*
            (?:
                (?:Q\s*)?(1|[Ff]irst|2|[Ss]econd|3|[Tt]hird|4|[Ff]ourth|[Ll]ast)(?:\s*quarter)?\s+([0-9]{4})
                |
                ([0-9]{4})\s+(?:Q\s*)?(1|[Ff]irst|2|[Ss]econd|3|[Tt]hird|4|[Ff]ourth|[Ll]ast)
            )
        [ \t\n\r]*$").unwrap()
    });

    let input = value.trim();

    if let Some(caps) = QUARTER_PATTERN.captures(input) {
        // Extract year and quarter from either pattern
        let year_str = caps
            .get(2)
            .or_else(|| caps.get(3))
            .ok_or_else(|| {
                TransformationError::InvalidInput(
                    value.to_string(),
                    "date_quarter_end: no year found".to_string(),
                )
            })?
            .as_str();

        let quarter_str = caps
            .get(1)
            .or_else(|| caps.get(4))
            .ok_or_else(|| {
                TransformationError::InvalidInput(
                    value.to_string(),
                    "date_quarter_end: no quarter found".to_string(),
                )
            })?
            .as_str()
            .to_lowercase();

        let year: i32 = year_str.parse().map_err(|_| {
            TransformationError::InvalidInput(
                value.to_string(),
                "date_quarter_end: invalid year".to_string(),
            )
        })?;

        // Map quarter to month
        let month = match quarter_str.as_str() {
            "first" | "1" => 3,
            "second" | "2" => 6,
            "third" | "3" => 9,
            "fourth" | "last" | "4" => 12,
            _ => {
                return Err(TransformationError::InvalidInput(
                    value.to_string(),
                    format!("date_quarter_end: invalid quarter: {}", quarter_str),
                ));
            }
        };

        // Calculate last day of the quarter month
        let day = match month {
            3 => {
                // March - check for leap year
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    // Leap year: Jan 31 + Feb 29 + Mar 31 = 91 days in Q1
                    31
                } else {
                    31
                }
            }
            6 => 30,  // June
            9 => 30,  // September
            12 => 31, // December
            _ => unreachable!(),
        };

        Ok(format!("{:04}-{:02}-{:02}", year, month, day))
    } else {
        Err(TransformationError::InvalidInput(
            value.to_string(),
            "date_quarter_end: no valid quarter pattern".to_string(),
        ))
    }
}

/// Converts US state/province names to 2-letter codes using SEC regex pattern
///
/// Supports all 50 US states, DC, US territories, and Canadian provinces.
///
/// ## Examples
/// - "Utah" → "UT"
/// - "Delaware" → "DE"
/// - "British Columbia" → "BC"
/// - "Puerto Rico" → "PR"
pub fn stateprovnameen(value: &str) -> Result<String, TransformationError> {
    static STATE_PROV_PATTERN: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?ix)^[ \t\n\r]*
            (
                ([Aa]labama)|([Aa]laska)|([Aa]rizona)|([Aa]rkansas)|([Cc]alifornia)|
                ([Cc]olorado)|([Cc]onnecticut)|([Dd]elaware)|([Ff]lorida)|([Gg]eorgia)|
                ([Hh]awaii)|([Ii]daho)|([Ii]llinois)|([Ii]ndiana)|([Ii]owa)|([Kk]ansas)|
                ([Kk]entucky)|([Ll]ouisiana)|([Mm]aine)|([Mm]aryland)|([Mm]assachusetts)|
                ([Mm]ichigan)|([Mm]innesota)|([Mm]ississippi)|([Mm]issouri)|([Mm]ontana)|
                ([Nn]ebraska)|([Nn]evada)|([Nn]ew\s+[Hh]ampshire)|([Nn]ew\s+[Jj]ersey)|
                ([Nn]ew\s+[Mm]exico)|([Nn]ew\s+[Yy]ork)|([Nn]orth\s+[Cc]arolina)|
                ([Nn]orth\s+[Dd]akota)|([Oo]hio)|([Oo]klahoma)|([Oo]regon)|
                ([Pp]ennsylvania)|([Rr]hode\s+[Ii]sland)|([Ss]outh\s+[Cc]arolina)|
                ([Ss]outh\s+[Dd]akota)|([Tt]ennessee)|([Tt]exas)|([Uu]tah)|([Vv]ermont)|
                ([Vv]irginia)|([Ww]ashington)|([Ww]est\s+[Vv]irginia)|([Ww]isconsin)|
                ([Ww]yoming)|([Dd]istrict\s+[Oo]f\s+[Cc]olumbia)|
                ((?:[Aa]merican\s+)?[Ss]amoa)|([Gg]uam)|([Nn]orthern\s+[Mm]ariana\s+[Ii]slands)|
                ([Pp]uerto\s+[Rr]ico)|([Uu]nited\s+[Ss]tates\s+[Mm]inor\s+[Oo]utlying\s+[Ii]slands)|
                ([Vv]irgin\s+[Ii]slands,\s+U[.]?S[.]?)|
                ([Aa]lberta)|([Bb]ritish\s+[Cc]olumbia)|([Mm]anitoba)|([Nn]ew\s+[Bb]runswick)|
                ([Nn]ewfoundland(?:\s+[Aa]nd\s+[Ll]abrador)?)|([Nn]ova\s+[Ss]cotia)|
                ([Nn]orthwest\s+[Tt]erritories)|([Nn]unavut)|([Oo]ntario)|
                ([Pp]rince\s+[Ee]dward\s+[Ii]sland)|([Qq]u[eé]bec)|([Ss]askatchewan)|([Yy]ukon)
            )
        [ \t\n\r]*$",
        )
        .unwrap()
    });

    // Codes corresponding to the capture groups above
    const STATE_PROV_CODES: &[&str] = &[
        "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
        "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT",
        "VA", "WA", "WV", "WI", "WY", "DC", "AS", "GU", "MP", "PR", "UM", "VI", "AB", "BC", "MB",
        "NB", "NL", "NS", "NT", "NU", "ON", "PE", "QC", "SK", "YT",
    ];

    let input = value.trim();

    if let Some(caps) = STATE_PROV_PATTERN.captures(input) {
        // Find which group matched (starting from group 2, since 1 is the outer group)
        for (idx, code) in STATE_PROV_CODES.iter().enumerate() {
            if caps.get(idx + 2).is_some() {
                return Ok(code.to_string());
            }
        }
    }

    Err(TransformationError::InvalidInput(
        value.to_string(),
        "stateprovnameen: no matching state/province".to_string(),
    ))
}

/// Converts duration days to ISO 8601 duration format
///
/// Handles decimal values with fractional spillover into hours.
/// If the input is not an integer, the fractional part spills into hours.
///
/// ## Examples
/// - "30" → "P30D"
/// - "30.5" → "P30DT12H" (0.5 days = 12 hours)
/// - "365" → "P365D"
/// - "-10.25" → "-P10DT6H"
pub fn dur_day(value: &str) -> Result<String, TransformationError> {
    let (n, sign) = parse_duration_value(value)?;
    let days = n.floor() as i64;
    let hours = ((n - days as f64) * 24.0).floor() as i64;
    format_duration(None, None, Some(days), Some(hours), sign)
}

/// Converts duration months to ISO 8601 duration format
///
/// Handles decimal values with fractional spillover into days.
/// Uses 30.4375 days per month for conversions.
///
/// ## Examples
/// - "12" → "P12M"
/// - "12.5" → "P12M15D" (0.5 months ≈ 15 days)
/// - "6" → "P6M"
/// - "-3.25" → "-P3M7D"
pub fn dur_month(value: &str) -> Result<String, TransformationError> {
    let (n, sign) = parse_duration_value(value)?;
    let months = n.floor() as i64;
    let days = ((n - months as f64) * 30.4375).floor() as i64;
    format_duration(None, Some(months), Some(days), None, sign)
}

/// Converts duration years to ISO 8601 duration format
///
/// Handles decimal values with fractional spillover into months and days.
///
/// ## Examples
/// - "1" → "P1Y"
/// - "1.5" → "P1Y6M" (0.5 years = 6 months)
/// - "2.1" → "P2Y1M9D"
pub fn dur_year(value: &str) -> Result<String, TransformationError> {
    let (n, sign) = parse_duration_value(value)?;
    let years = n.floor() as i64;
    let months_decimal = (n - years as f64) * 12.0;
    let months = months_decimal.floor() as i64;
    let days = ((months_decimal - months as f64) * 30.4375).floor() as i64;
    format_duration(Some(years), Some(months), Some(days), None, sign)
}

/// Converts duration weeks to ISO 8601 duration format
///
/// Converts weeks to days (xs:duration doesn't have weeks).
///
/// ## Examples
/// - "1" → "P7D"
/// - "2.5" → "P17D" (2.5 weeks = 17.5 days, truncated)
pub fn dur_week(value: &str) -> Result<String, TransformationError> {
    let (n, sign) = parse_duration_value(value)?;
    let days = (n * 7.0).floor() as i64;
    format_duration(None, None, Some(days), None, sign)
}

/// Converts duration hours to ISO 8601 duration format
///
/// ## Examples
/// - "24" → "PT24H"
/// - "1.5" → "PT1H" (fractional hours truncated)
pub fn dur_hour(value: &str) -> Result<String, TransformationError> {
    let (n, sign) = parse_duration_value(value)?;
    let hours = n.floor() as i64;
    format_duration(None, None, None, Some(hours), sign)
}

/// Helper to parse a duration value and extract sign
fn parse_duration_value(value: &str) -> Result<(f64, &'static str), TransformationError> {
    static DECIMAL_PATTERN: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[ \t\n\r]*([+-])?([0-9]+(\.[0-9]*)?|\.[0-9]+)[ \t\n\r]*$").unwrap()
    });

    let input = value.trim();
    if let Some(caps) = DECIMAL_PATTERN.captures(input) {
        let sign_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let num_str = caps.get(2).unwrap().as_str();

        let n: f64 = num_str.parse().map_err(|_| {
            TransformationError::InvalidInput(value.to_string(), "duration value".to_string())
        })?;

        let sign = if sign_str == "-" { "-" } else { "" };
        let abs_n = n.abs();

        Ok((abs_n, sign))
    } else {
        Err(TransformationError::InvalidInput(
            value.to_string(),
            "duration value (expected decimal number)".to_string(),
        ))
    }
}

/// Helper to format ISO 8601 duration
fn format_duration(
    years: Option<i64>,
    months: Option<i64>,
    days: Option<i64>,
    hours: Option<i64>,
    sign: &str,
) -> Result<String, TransformationError> {
    // Handle special case: all zeros
    let all_zero = years.unwrap_or(0) == 0
        && months.unwrap_or(0) == 0
        && days.unwrap_or(0) == 0
        && hours.unwrap_or(0) == 0;

    if all_zero {
        // Return P0Y, P0M, P0D, or PT0H depending on which components are present
        if years.is_some() {
            return Ok("P0Y".to_string());
        } else if months.is_some() {
            return Ok("P0M".to_string());
        } else if days.is_some() {
            return Ok("P0D".to_string());
        } else if hours.is_some() {
            return Ok("PT0H".to_string());
        }
        return Ok("P0D".to_string());
    }

    let mut result = format!("{}P", sign);

    // Add year/month/day components
    if let Some(y) = years {
        if y != 0 {
            result.push_str(&format!("{}Y", y));
        }
    }

    if let Some(m) = months {
        if m != 0 || years.is_none() {
            result.push_str(&format!("{}M", m));
        }
    }

    if let Some(d) = days {
        if d != 0 || (years.is_none() && months.is_none()) {
            result.push_str(&format!("{}D", d));
        }
    }

    // Add time component if hours present
    if let Some(h) = hours {
        if h != 0 || (years.is_none() && months.is_none() && days.is_none()) {
            result.push_str(&format!("T{}H", h));
        }
    }

    Ok(result)
}

/// Converts entity filer category to standardized format using SEC regex pattern
///
/// Normalizes variations of filer categories to standard strings.
///
/// ## Examples
/// - "Large Accelerated Filer" → "Large Accelerated Filer"
/// - "Non-accelerated Filer" → "Non-accelerated Filer"  
/// - "Non accelerated Filer" → "Non-accelerated Filer"
pub fn entityfilercategoryen(value: &str) -> Result<String, TransformationError> {
    static ENTITY_FILER_PATTERN: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?ix)^[ \t\n\r]*
            (
                ([Ll]arge\s+[Aa]ccelerated\s+[Ff]iler)|
                ([Aa]ccelerated\s+[Ff]iler)|
                ([Nn]on[^\w]+[Aa]ccelerated\s+[Ff]iler)
            )
        [ \t\n\r]*$",
        )
        .unwrap()
    });

    const ENTITY_FILER_CODES: &[&str] = &[
        "Large Accelerated Filer",
        "Accelerated Filer",
        "Non-accelerated Filer",
    ];

    let input = value.trim();

    if let Some(caps) = ENTITY_FILER_PATTERN.captures(input) {
        for (idx, code) in ENTITY_FILER_CODES.iter().enumerate() {
            if caps.get(idx + 2).is_some() {
                return Ok(code.to_string());
            }
        }
    }

    Err(TransformationError::InvalidInput(
        value.to_string(),
        "entityfilercategoryen: no matching filer category".to_string(),
    ))
}

/// Converts EDGAR province/country codes to standard format
///
/// ## Examples
/// - "CAYMAN ISLANDS" → "Cayman Islands"
/// - "DELAWARE" → "Delaware"
pub fn edgarprovcountryen(value: &str) -> Result<String, TransformationError> {
    // Title case the input
    let words: Vec<String> = value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect();
    Ok(words.join(" "))
}

/// Converts country names to 2-letter ISO country codes
///
/// Supports 246 countries with many name variations.
/// Based on SEC EDGAR specification using regex capture group indexing.
///
/// ## Examples
/// - "United States" → "US"
/// - "United Kingdom" → "GB"
/// - "People's Republic of China" → "CN"
pub fn countrynameen(value: &str) -> Result<String, TransformationError> {
    // Helper function to find which capture group matched
    fn find_code_index(caps: &regex::Captures) -> Option<usize> {
        // Skip group 0 (full match) and group 1 (outer group)
        for idx in 2..caps.len() {
            if caps.get(idx).is_some() {
                return Some(idx - 2); // Convert to 0-based index
            }
        }
        None
    }

    static COUNTRY_PATTERN: Lazy<Regex> = Lazy::new(|| {
        // This is a simplified version - the full SEC pattern has 246 country patterns
        // Using non-capturing groups (?:...) for optional parts to avoid indexing issues
        Regex::new(r"(?ix)^[ \t\n\r]*(
            ([Aa]fghanistan)|([ÅåAa]land\s+[Ii]slands)|([Aa]lbania)|([Aa]lgeria)|([Aa]merican\s+[Ss]amoa)|
            ([Aa]ndorra)|([Aa]ngola)|([Aa]nguilla)|([Aa]ntarctica)|([Aa]ntigua\s+[Aa]nd\s+[Bb]arbuda)|
            ((?:(?:[Tt]he\s+)?[Rr]epublic\s+[Oo]f\s+)?[Aa]rgentina)|([Aa]rmenia)|([Aa]ruba)|([Aa]ustralia)|
            ([Aa]ustria)|([Aa]zerbaijan)|([Bb]ahamas)|([Bb]ahrain)|([Bb]angladesh)|([Bb]arbados)|
            ([Bb]elarus)|([Bb]elgium)|([Bb]elize)|([Bb]enin)|([Bb]ermuda)|([Bb]hutan)|([Bb]olivia)|
            ([Bb]onaire,\s+[Ss]int\s+[Ee]ustatius\s+[Aa]nd\s+[Ss]aba)|([Bb]osnia\s+[Aa]nd\s+[Hh]erzegovina)|
            ([Bb]otswana)|([Bb]ouvet\s+[Ii]sland)|((?:(?:[Tt]he\s+)?[Ff]ederative\s+[Rr]epublic\s+[Oo]f\s+)?[Bb]ra[sz]il)|
            ([Bb]ritish\s+[Ii]ndian\s+[Oo]cean\s+[Tt]erritory)|([Bb]runei\s+[Dd]arussalam)|([Bb]ulgaria)|
            ([Bb]urkina\s+[Ff]aso)|([Bb]urundi)|([Cc]abo\s+[Vv]erde)|([Cc]ambodia)|([Cc]ameroon)|([Cc]anada)|
            ([Cc]ayman\s+[Ii]slands)|([Cc]entral\s+[Aa]frican\s+[Rr]epublic)|([Cc]had)|([Cc]hile)|
            ((?:(?:[Tt]he\s+)?[Pp]eople['']?s\s+[Rr]epublic\s+[Oo]f\s+)?[Cc]hina)|([Cc]hristmas\s+[Ii]sland)|
            ([Cc]ocos\s+(?:.[Kk]eeling.\s+)?[Ii]slands)|((?:(?:[Tt]he\s+)?[Rr]epublic\s+[Oo]f\s+)?[Cc]olombia)|
            ([Cc]omoros)|([Dd]emocratic\s+[Rr]epublic\s+[Oo]f\s+(?:[Tt]he\s+)?[Cc]ongo)|([Cc]ongo)|
            ([Cc]ook\s+[Ii]slands)|([Cc]osta\s+[Rr]ica)|([Cc][ôo]te\s+d[''][Ii]voire)|([Cc]roatia)|([Cc]uba)|
            ([Cc]ura[çc]ao)|([Cc]yprus)|([Cc]zechia|[Cc]zech\s+[Rr]epublic)|((?:(?:[Tt]he\s+)?[Kk]ingdom\s+[Oo]f\s+)?[Dd]enmark)|
            ([Dd]jibouti)|([Dd]ominica)|([Dd]ominican\s+[Rr]epublic)|([Ee]cuador)|([Ee]gypt)|([Ee]l\s+[Ss]alvador)|
            ([Ee]quatorial\s+[Gg]uinea)|([Ee]ritrea)|([Ee]stonia)|([Ee]swatini)|([Ee]thiopia)|
            ((?:[Tt]he\s+)?[Ff]alkland\s+[Ii]slands(?:\s+.[Mm]alvinas.)?|(?:[Ii]slas\s+)?[Mm]alvinas(?:\s+[Ii]slands)?)|
            ([Ff]aroe\s+[Ii]slands)|([Ff]iji)|([Ff]inland)|([Ff]rance)|([Ff]rench\s+[Gg]uiana)|
            ([Ff]rench\s+[Pp]olynesia)|([Ff]rench\s+[Ss]outhern\s+[Tt]erritories)|([Gg]abon)|([Gg]ambia)|
            ([Gg]eorgia)|([Gg]ermany)|([Gg]hana)|([Gg]ibraltar)|([Gg]reece)|([Gg]reenland)|([Gg]renada)|
            ([Gg]uadeloupe)|([Gg]uam)|([Gg]uatemala)|([Gg]uernsey)|([Gg]uinea)|([Gg]uinea-[Bb]issau)|
            ([Gg]uyana)|([Hh]aiti)|([Hh]eard\s+[Ii]sland\s+[Aa]nd\s+[Mm]cDonald\s+[Ii]slands)|
            ((?:[Tt]he\s+)?[Hh]oly\s+[Ss]ee|[Vv]atican\s+[Cc]ity)|([Hh]onduras)|([Hh]ong\s+[Kk]ong)|
            ([Hh]ungary)|([Ii]celand)|([Ii]ndia)|([Ii]ndonesia)|((?:(?:[Tt]he\s+)?[Ii]slamic\s+[Rr]epublic\s+of\s+)?[Ii]ran)|
            ([Ii]raq)|([Ii]reland)|([Ii]sle\s+[Oo]f\s+[Mm]an)|([Ii]srael)|([Ii]taly)|([Jj]amaica)|([Jj]apan)|
            ([Jj]ersey)|([Jj]ordan)|([Kk]azakhstan)|([Kk]enya)|([Kk]iribati)|
            ((?:[Nn]orth|[Dd]emocratic\s+[Pp]eople['']?s\s+[Rr]epublic\s+[Oo]f)\s+[Kk]orea)|
            ((?:[Ss]outh|[Rr]epublic\s+[Oo]f)\s+[Kk]orea)|([Kk]uwait)|([Kk]yrgyzstan)|
            ([Ll]ao\s+[Pp]eople['']?s\s+[Dd]emocratic\s+[Rr]epublic)|([Ll]atvia)|([Ll]ebanon)|([Ll]esotho)|
            ([Ll]iberia)|((?:(?:[Tt]he\s+)?[Ss]tate\s+[Oo]f\s+)?[Ll]ibya)|([Ll]iechtenstein)|([Ll]ithuania)|
            ((?:(?:[Tt]he\s+)?(?:[Gg]rand\s+)?[Dd]uchy\s+[Oo]f\s+)?[Ll]uxembourg)|([Mm]aca[ou])|([Mm]adagascar)|
            ([Mm]alawi)|([Mm]alaysia)|([Mm]aldives)|([Mm]ali)|([Mm]alta)|
            ((?:(?:[Tt]he\s+)?[Rr]epublic\s+[Oo]f\s+(?:[Tt]he\s+)?)?[Mm]arshall\s+[Ii]slands)|([Mm]artinique)|
            ([Mm]auritania)|([Mm]auritius)|([Mm]ayotte)|([Mm]exico|[Uu]nited\s+[Mm]exican\s+[Ss]tates)|
            ((?:(?:[Tt]he\s+)?[Ff]ederated\s+[Ss]tates\s+[Oo]f\s+)?[Mm]icronesia)|((?:(?:[Tt]he\s+)?[Rr]epublic\s+[Oo]f\s+)?[Mm]oldova)|
            ([Mm]onaco)|([Mm]ongolia)|([Mm]ontenegro)|([Mm]ontserrat)|([Mm]orocco)|([Mm]ozambique)|
            ([Mm]yanmar)|([Nn]amibia)|([Nn]auru)|([Nn]epal)|([Nn]etherlands)|([Nn]ew\s+[Cc]aledonia)|
            ([Nn]ew\s+[Zz]ealand)|([Nn]icaragua)|([Nn]iger)|([Nn]igeria)|([Nn]iue)|([Nn]orfolk\s+[Ii]sland)|
            ((?:(?:[Tt]he\s+)?[Rr]epublic\s+[Oo]f\s+)?[Nn]orth\s+[Mm]acedonia)|([Nn]orthern\s+[Mm]ariana\s+[Ii]slands)|
            ([Nn]orway)|([Oo]man)|([Pp]akistan)|([Pp]alau)|((?:(?:[Tt]he\s+)?[Ss]tate\s+[Oo]f\s+)?[Pp]alestine)|
            ([Pp]anama)|([Pp]apua\s+[Nn]ew\s+[Gg]uinea)|([Pp]araguay)|([Pp]eru)|([Pp]hilippines)|([Pp]itcairn)|
            ([Pp]oland)|([Pp]ortugal)|([Pp]uerto\s+[Rr]ico)|([Qq]atar)|([Rr][ée]union)|([Rr]omania)|
            ([Rr]ussian\s+[Ff]ederation)|([Rr]wanda)|([Ss]aint\s+[Bb]arth[ée]lemy)|
            ([Ss]aint\s+[Hh]elena(?:,\s+[Aa]scension,?\s+[Aa]nd\s+[Tt]ristan(?:\s+[Dd]a\s+[Cc]unha)?)?)|
            ([Ss]aint\s+[Kk]itts\s+[Aa]nd\s+[Nn]evis)|([Ss]aint\s+[Ll]ucia)|([Ss]aint\s+[Mm]artin)|
            ([Ss]aint\s+[Pp]ierre\s+[Aa]nd\s+[Mm]iquelon)|([Ss]aint\s+[Vv]incent(?:\s+[Aa]nd\s+(?:[Tt]he\s+)?[Gg]renadines)?)|
            ([Ss]amoa)|([Ss]an\s+[Mm]arino)|([Ss]ao\s+[Tt]ome\s+[Aa]nd\s+[Pp]rincipe)|([Ss]audi\s+[Aa]rabia)|
            ([Ss]enegal)|([Ss]erbia)|([Ss]eychelles)|([Ss]ierra\s+[Ll]eone)|([Ss]ingapore)|([Ss]int\s+[Mm]aarten)|
            ([Ss]lovakia)|([Ss]lovenia)|([Ss]olomon\s+[Ii]slands)|([Ss]omalia)|
            ((?:(?:[Tt]he\s+)?[Rr]epublic\s+[Oo]f\s+)?[Ss]outh\s+[Aa]frica)|
            ([Ss]outh\s+[Gg]eorgia\s+[Aa]nd\s+(?:[Tt]he\s+)?[Ss]outh\s+[Ss]andwich\s+[Ii]slands)|
            ([Ss]outh\s+[Ss]udan)|((?:(?:[Tt]he\s+)?[Kk]ingdom\s+[Oo]f\s+)?[Ss]pain|[Ee]spa[ñn]a)|
            ([Ss]ri\s+[Ll]anka)|([Ss]udan)|([Ss]uriname)|([Ss]valbard\s+[Aa]nd\s+[Jj]an\s+[Mm]ayen)|
            ([Ss]weden)|([Ss]witzerland)|([Ss]yria(?:n\s+[Aa]rab\s+[Rr]epublic)?)|([Tt]aiwan(?:,?\s+[Pp]rovince\s+[Oo]f\s+[Cc]hina)?)|
            ([Tt]ajikistan)|((?:(?:[Tt]he\s+)?[Uu]nited\s+[Rr]epublic\s+[Oo]f\s+)?[Tt]anzania)|([Tt]hailand)|
            ([Tt]imor-[Ll]este)|([Tt]ogo)|([Tt]okelau)|([Tt]onga)|([Tt]rinidad\s+[Aa]nd\s+[Tt]obago)|
            ([Tt]unisia)|([Tt]urkey)|([Tt]urkmenistan)|([Tt]urks\s+[Aa]nd\s+[Cc]aicos\s+[Ii]slands)|
            ([Tt]uvalu)|([Uu]ganda)|([Uu]kraine)|
            (UAE|[Uu]nited\s+[Aa]rab\s+[Ee]mirates)|
            (U[.]?K[.]?|[Bb]ritain|[Gg]reat\s+[BBb]ritain|[Uu]nited\s+[Kk]ingdom(?:\s+[Oo]f\s+[Gg]reat\s+[Bb]ritain\s+[Aa]nd\s+[Nn]orthern\s+[Ii]reland)?|[Ee]ngland(?:\s+[Aa]nd\s+[Ww]ales)?)|
            ([Uu]nited\s+[Ss]tates\s+[Mm]inor\s+[Oo]utlying\s+[Ii]slands)|
            (U[.]?S[.]?A[.]?|[Uu]nited\s+[Ss]tates(?:\s+[Oo]f\s+[Aa]merica)?)|
            ([Uu]ruguay)|([Uu]zbekistan)|([Vv]anuatu)|([Vv]enezuela)|([Vv]iet\s+[Nn]am)|
            ([Bb]ritish\s+[Vv]irgin\s+[Ii]slands)|([Uu][.]?[Ss][.]?\s+[Vv]irgin\s+[Ii]slands)|
            ([Ww]allis\s+[Aa]nd\s+[Ff]utuna)|([Ww]estern\s+[Ss]ahara)|([Yy]emen)|([Zz]ambia)|([Zz]imbabwe)
        )[ \t\n\r]*$").unwrap()
    });

    // ISO country codes in order matching the regex patterns
    const COUNTRY_CODES: &[&str] = &[
        "AF", "AX", "AL", "DZ", "AS", "AD", "AO", "AI", "AQ", "AG", "AR", "AM", "AW", "AU", "AT",
        "AZ", "BS", "BH", "BD", "BB", "BY", "BE", "BZ", "BJ", "BM", "BT", "BO", "BQ", "BA", "BW",
        "BV", "BR", "IO", "BN", "BG", "BF", "BI", "CV", "KH", "CM", "CA", "KY", "CF", "TD", "CL",
        "CN", "CX", "CC", "CO", "KM", "CD", "CG", "CK", "CR", "CI", "HR", "CU", "CW", "CY", "CZ",
        "DK", "DJ", "DM", "DO", "EC", "EG", "SV", "GQ", "ER", "EE", "SZ", "ET", "FK", "FO", "FJ",
        "FI", "FR", "GF", "PF", "TF", "GA", "GM", "GE", "DE", "GH", "GI", "GR", "GL", "GD", "GP",
        "GU", "GT", "GG", "GN", "GW", "GY", "HT", "HM", "VA", "HN", "HK", "HU", "IS", "IN", "ID",
        "IR", "IQ", "IE", "IM", "IL", "IT", "JM", "JP", "JE", "JO", "KZ", "KE", "KI", "KP", "KR",
        "KW", "KG", "LA", "LV", "LB", "LS", "LR", "LY", "LI", "LT", "LU", "MO", "MG", "MW", "MY",
        "MV", "ML", "MT", "MH", "MQ", "MR", "MU", "YT", "MX", "FM", "MD", "MC", "MN", "ME", "MS",
        "MA", "MZ", "MM", "NA", "NR", "NP", "NL", "NC", "NZ", "NI", "NE", "NG", "NU", "NF", "MK",
        "MP", "NO", "OM", "PK", "PW", "PS", "PA", "PG", "PY", "PE", "PH", "PN", "PL", "PT", "PR",
        "QA", "RE", "RO", "RU", "RW", "BL", "SH", "KN", "LC", "MF", "PM", "VC", "WS", "SM", "ST",
        "SA", "SN", "RS", "SC", "SL", "SG", "SX", "SK", "SI", "SB", "SO", "ZA", "GS", "SS", "ES",
        "LK", "SD", "SR", "SJ", "SE", "CH", "SY", "TW", "TJ", "TZ", "TH", "TL", "TG", "TK", "TO",
        "TT", "TN", "TR", "TM", "TC", "TV", "UG", "UA", "AE", "GB", "UM", "US", "UY", "UZ", "VU",
        "VE", "VN", "VG", "VI", "WF", "EH", "YE", "ZM", "ZW",
    ];

    let input = value.trim();

    if let Some(caps) = COUNTRY_PATTERN.captures(input) {
        if let Some(idx) = find_code_index(&caps) {
            if idx < COUNTRY_CODES.len() {
                return Ok(COUNTRY_CODES[idx].to_string());
            }
        }
    }

    Err(TransformationError::InvalidInput(
        value.to_string(),
        "countrynameen: no matching country".to_string(),
    ))
}

// --- Registries ---

// SEC-specific transformations (ixt-sec)
static IXT_SEC_REGISTRY: Lazy<TransformationRegistry> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("boolballotbox", bool_ballot_box as Transformation);
    m.insert("yesnoballotbox", yesno_ballot_box as Transformation);
    m.insert("numwordsen", num_words_en as Transformation);
    m.insert("durwordsen", dur_words_en as Transformation);
    m.insert("duryear", dur_year as Transformation);
    m.insert("durmonth", dur_month as Transformation);
    m.insert("durweek", dur_week as Transformation);
    m.insert("durday", dur_day as Transformation);
    m.insert("durhour", dur_hour as Transformation);
    m.insert("datequarterend", date_quarter_end as Transformation);
    m.insert("numinf", num_inf as Transformation);
    m.insert("numneginf", num_neg_inf as Transformation);
    m.insert("numnan", num_nan as Transformation);
    m.insert("exchnameen", exchnameen as Transformation);
    m.insert("stateprovnameen", stateprovnameen as Transformation);
    m.insert("countrynameen", countrynameen as Transformation);
    m.insert(
        "entityfilercategoryen",
        entityfilercategoryen as Transformation,
    );
    m.insert("edgarprovcountryen", edgarprovcountryen as Transformation);
    m
});

// Standard transformations (ixt namespace, v4 registry)
static IXT_REGISTRY_V4: Lazy<TransformationRegistry> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("fixed-false", fixed_false as Transformation);
    m.insert("fixed-true", fixed_true as Transformation);
    m.insert("fixed-zero", fixed_zero as Transformation);
    m.insert("zerodash", zero_dash as Transformation);
    m.insert("zero-dash", zero_dash as Transformation); // Alternative spelling
    m.insert("num-dot-decimal", num_dot_decimal as Transformation);
    m.insert("numdotdecimal", num_dot_decimal as Transformation); // Spelling variant
    m.insert("booleanfalse", boolean_false as Transformation);
    m.insert("booleantrue", boolean_true as Transformation);
    m.insert(
        "date-monthname-day-year-en",
        date_month_day_year_en as Transformation,
    );
    m.insert(
        "datemonthdayyearen",
        date_month_day_year_en as Transformation,
    ); // Alternative spelling
    m
});

/// Applies a transformation to a value based on its format string.
///
/// The format string is expected to be in the format `prefix:localname`,
/// where prefix identifies the transformation registry (e.g., "ixt-sec", "ixt").
///
/// ## Arguments
/// - `value`: The raw text value from the iXBRL document
/// - `format`: The transformation format (e.g., "ixt-sec:numwordsen")
///
/// ## Returns
/// - `Ok(String)`: The normalized value
/// - `Err(TransformationError)`: If transformation fails
///
/// ## Examples
/// ```
/// use xbrl::transformations::apply_transformation;
///
/// let result = apply_transformation("one", "ixt-sec:numwordsen");
/// assert_eq!(result.unwrap(), "1");
///
/// let result = apply_transformation("1,000", "ixt:num-dot-decimal");
/// assert_eq!(result.unwrap(), "1000");
/// ```
pub fn apply_transformation(value: &str, format: &str) -> Result<String, TransformationError> {
    let parts: Vec<&str> = format.split(':').collect();
    if parts.len() != 2 {
        // Not a namespaced format, return original value
        return Ok(value.to_string());
    }
    let prefix = parts[0];
    let name = parts[1];

    let registry = match prefix {
        "ixt-sec" => &*IXT_SEC_REGISTRY,
        "ixt" => &*IXT_REGISTRY_V4,
        _ => return Ok(value.to_string()), // Unknown registry, pass through
    };

    if let Some(transformer) = registry.get(name) {
        transformer(value)
    } else {
        // Transformation not found in the registry, pass through
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_ballot_box() {
        assert_eq!(bool_ballot_box("☐").unwrap(), "false");
        assert_eq!(bool_ballot_box("☑").unwrap(), "true");
        assert_eq!(bool_ballot_box("☒").unwrap(), "true");
        assert!(bool_ballot_box("invalid").is_err());
    }

    #[test]
    fn test_num_words_en_simple() {
        assert_eq!(num_words_en("one").unwrap(), "1");
        assert_eq!(num_words_en("zero").unwrap(), "0");
        assert_eq!(num_words_en("twelve").unwrap(), "12");
        assert_eq!(num_words_en("ninety nine").unwrap(), "99");

        // Test special cases that map to zero (matching Python's numwordsNoPattern)
        assert_eq!(num_words_en("no").unwrap(), "0");
        assert_eq!(num_words_en("none").unwrap(), "0");
        assert_eq!(num_words_en("nil").unwrap(), "0");
        assert_eq!(num_words_en("No").unwrap(), "0");
        assert_eq!(num_words_en("NONE").unwrap(), "0");
        assert_eq!(num_words_en("Nil").unwrap(), "0");
    }

    #[test]
    fn test_num_words_en_complex() {
        assert_eq!(num_words_en("seventy thousand and one").unwrap(), "70001");
        assert_eq!(num_words_en("nineteen hundred forty-four").unwrap(), "1944");
        assert_eq!(
            num_words_en("six million four hundred thousand five").unwrap(),
            "6400005"
        );
    }

    #[test]
    fn test_dur_words_en() {
        // Basic tests
        assert_eq!(dur_words_en("9 years, 2 months").unwrap(), "P9Y2M");
        assert_eq!(dur_words_en("Five years, two months").unwrap(), "P5Y2M");
        assert_eq!(
            dur_words_en("three years four months no days").unwrap(),
            "P3Y4M"
        );

        // Test with "and"
        assert_eq!(dur_words_en("5 years and 3 months").unwrap(), "P5Y3M");
        assert_eq!(
            dur_words_en("10 years, 6 months and 15 days").unwrap(),
            "P10Y6M15D"
        );

        // Test "no", "zero", "none", "nil" as zero values
        assert_eq!(dur_words_en("no years, 5 months").unwrap(), "P5M");
        assert_eq!(
            dur_words_en("zero years, zero months, 10 days").unwrap(),
            "P10D"
        );
        assert_eq!(dur_words_en("none years, 3 months").unwrap(), "P3M");
        assert_eq!(
            dur_words_en("nil years, nil months, 7 days").unwrap(),
            "P7D"
        );

        // Test Unicode dashes (Armenian hyphen U+058A)
        assert_eq!(dur_words_en("3֊years").unwrap(), "P3Y");
        // Hebrew maqaf (U+05BE)
        assert_eq!(dur_words_en("2־months").unwrap(), "P2M");
        // En dash (U+2013)
        assert_eq!(dur_words_en("5–years").unwrap(), "P5Y");
        // Em dash (U+2014)
        assert_eq!(dur_words_en("7—months").unwrap(), "P7M");
        // Small em dash (U+FE58)
        assert_eq!(dur_words_en("4﹘days").unwrap(), "P4D");
        // Fullwidth hyphen-minus (U+FF0D)
        assert_eq!(dur_words_en("9－years").unwrap(), "P9Y");

        // Test all zeros returns P0D
        assert_eq!(dur_words_en("no years, no months, no days").unwrap(), "P0D");
        assert_eq!(dur_words_en("zero years").unwrap(), "P0D");
    }

    #[test]
    fn test_num_dot_decimal() {
        assert_eq!(num_dot_decimal("1,000,000").unwrap(), "1000000");
        assert_eq!(num_dot_decimal("123,456.789").unwrap(), "123456.789");
        assert_eq!(num_dot_decimal("42").unwrap(), "42");
    }

    #[test]
    fn test_zero_dash() {
        // Standard hyphen-minus
        assert_eq!(zero_dash("-").unwrap(), "0");

        // Em dash
        assert_eq!(zero_dash("—").unwrap(), "0");

        // En dash
        assert_eq!(zero_dash("–").unwrap(), "0");

        // With whitespace
        assert_eq!(zero_dash("  -  ").unwrap(), "0");
        assert_eq!(zero_dash("\t-\t").unwrap(), "0");

        // Non-dash values should pass through
        assert_eq!(zero_dash("123").unwrap(), "123");
        assert_eq!(zero_dash("0").unwrap(), "0");
        assert_eq!(zero_dash("  456  ").unwrap(), "456"); // trimmed
    }

    #[test]
    fn test_apply_transformation() {
        assert_eq!(
            apply_transformation("one", "ixt-sec:numwordsen").unwrap(),
            "1"
        );
        assert_eq!(
            apply_transformation("☐", "ixt-sec:boolballotbox").unwrap(),
            "false"
        );
        assert_eq!(
            apply_transformation("1,000", "ixt:num-dot-decimal").unwrap(),
            "1000"
        );
        // Unknown transformation should pass through
        assert_eq!(
            apply_transformation("value", "unknown:transform").unwrap(),
            "value"
        );
    }

    #[test]
    fn test_boolean_transformations() {
        assert_eq!(boolean_false("anything").unwrap(), "false");
        assert_eq!(boolean_true("anything").unwrap(), "true");
        assert_eq!(
            apply_transformation("x", "ixt:booleanfalse").unwrap(),
            "false"
        );
        assert_eq!(
            apply_transformation("x", "ixt:booleantrue").unwrap(),
            "true"
        );
    }

    #[test]
    fn test_exchnameen() {
        assert_eq!(exchnameen("New York Stock Exchange").unwrap(), "NYSE");
        assert_eq!(exchnameen("The New York Stock Exchange").unwrap(), "NYSE");
        assert_eq!(exchnameen("The NYSE").unwrap(), "NYSE");
        assert_eq!(
            exchnameen("The NASDAQ Stock Market, LLC").unwrap(),
            "NASDAQ"
        );
        assert_eq!(exchnameen("NASDAQ Stock Market").unwrap(), "NASDAQ");
        assert_eq!(exchnameen("Cboe BZX Exchange, Inc.").unwrap(), "CboeBZX");
        // Test that invalid exchanges fail
        assert!(exchnameen("Other Exchange").is_err());
    }

    #[test]
    fn test_date_month_day_year_en() {
        assert_eq!(
            date_month_day_year_en("August 22, 2025").unwrap(),
            "2025-08-22"
        );
        assert_eq!(
            date_month_day_year_en("November 29, 2021").unwrap(),
            "2021-11-29"
        );
        assert_eq!(
            date_month_day_year_en("January 1, 2020").unwrap(),
            "2020-01-01"
        );
        assert_eq!(
            date_month_day_year_en("December 31, 2023").unwrap(),
            "2023-12-31"
        );
        // Test both registry names
        assert_eq!(
            apply_transformation("August 22, 2025", "ixt:date-monthname-day-year-en").unwrap(),
            "2025-08-22"
        );
        assert_eq!(
            apply_transformation("August 22, 2025", "ixt:datemonthdayyearen").unwrap(),
            "2025-08-22"
        );
    }

    #[test]
    fn test_stateprovnameen() {
        assert_eq!(stateprovnameen("Utah").unwrap(), "UT");
        assert_eq!(stateprovnameen("Delaware").unwrap(), "DE");
        assert_eq!(stateprovnameen("California").unwrap(), "CA");
        assert_eq!(stateprovnameen("New York").unwrap(), "NY");
        // Canadian provinces
        assert_eq!(stateprovnameen("Ontario").unwrap(), "ON");
        assert_eq!(stateprovnameen("British Columbia").unwrap(), "BC");
        assert_eq!(stateprovnameen("Quebec").unwrap(), "QC");
        assert_eq!(stateprovnameen("Québec").unwrap(), "QC");
        // Territories
        assert_eq!(stateprovnameen("Puerto Rico").unwrap(), "PR");
        // Test that invalid state names fail
        assert!(stateprovnameen("Cayman Islands").is_err());
    }

    #[test]
    fn test_dur_day() {
        assert_eq!(dur_day("30").unwrap(), "P30D");
        assert_eq!(dur_day("365").unwrap(), "P365D");
        assert_eq!(dur_day("1").unwrap(), "P1D");
        assert_eq!(dur_day("30.5").unwrap(), "P30DT12H");
    }

    #[test]
    fn test_dur_month() {
        assert_eq!(dur_month("12").unwrap(), "P12M");
        assert_eq!(dur_month("6").unwrap(), "P6M");
        assert_eq!(dur_month("1").unwrap(), "P1M");
        assert_eq!(dur_month("12.5").unwrap(), "P12M15D");
    }

    #[test]
    fn test_entityfilercategoryen() {
        assert_eq!(
            entityfilercategoryen("Large Accelerated Filer").unwrap(),
            "Large Accelerated Filer"
        );
        assert_eq!(
            entityfilercategoryen("  Non-accelerated Filer  ").unwrap(),
            "Non-accelerated Filer"
        );
    }

    #[test]
    fn test_edgarprovcountryen() {
        assert_eq!(
            edgarprovcountryen("CAYMAN ISLANDS").unwrap(),
            "Cayman Islands"
        );
        assert_eq!(edgarprovcountryen("DELAWARE").unwrap(), "Delaware");
        assert_eq!(edgarprovcountryen("NEW YORK").unwrap(), "New York");
    }

    #[test]
    fn test_numdotdecimal_alias() {
        // Test that both spellings work
        assert_eq!(
            apply_transformation("1,234.56", "ixt:num-dot-decimal").unwrap(),
            "1234.56"
        );
        assert_eq!(
            apply_transformation("1,234.56", "ixt:numdotdecimal").unwrap(),
            "1234.56"
        );
    }

    #[test]
    fn test_date_quarter_end() {
        // Q notation
        assert_eq!(date_quarter_end("Q1 2023").unwrap(), "2023-03-31");
        assert_eq!(date_quarter_end("Q2 2023").unwrap(), "2023-06-30");
        assert_eq!(date_quarter_end("Q3 2023").unwrap(), "2023-09-30");
        assert_eq!(date_quarter_end("Q4 2023").unwrap(), "2023-12-31");

        // Number notation
        assert_eq!(date_quarter_end("1 2023").unwrap(), "2023-03-31");
        assert_eq!(date_quarter_end("2 2023").unwrap(), "2023-06-30");
        assert_eq!(date_quarter_end("3 2023").unwrap(), "2023-09-30");
        assert_eq!(date_quarter_end("4 2023").unwrap(), "2023-12-31");

        // Word notation
        assert_eq!(date_quarter_end("first 2023").unwrap(), "2023-03-31");
        assert_eq!(date_quarter_end("second 2023").unwrap(), "2023-06-30");
        assert_eq!(date_quarter_end("third 2023").unwrap(), "2023-09-30");
        assert_eq!(date_quarter_end("fourth 2023").unwrap(), "2023-12-31");
        assert_eq!(date_quarter_end("last 2023").unwrap(), "2023-12-31");

        // Reverse order (year first)
        assert_eq!(date_quarter_end("2023 Q2").unwrap(), "2023-06-30");
        assert_eq!(date_quarter_end("2023 second").unwrap(), "2023-06-30");
        assert_eq!(date_quarter_end("2023 3").unwrap(), "2023-09-30");
    }

    #[test]
    fn test_yesno_ballot_box() {
        assert_eq!(yesno_ballot_box("☐").unwrap(), "No");
        assert_eq!(yesno_ballot_box("☑").unwrap(), "Yes");
        assert_eq!(yesno_ballot_box("☒").unwrap(), "Yes");
        assert!(yesno_ballot_box("invalid").is_err());
        assert!(yesno_ballot_box("").is_err());
    }

    #[test]
    fn test_special_floats() {
        assert_eq!(num_inf("anything").unwrap(), "INF");
        assert_eq!(num_neg_inf("anything").unwrap(), "-INF");
        assert_eq!(num_nan("anything").unwrap(), "NaN");

        // Test via registry
        assert_eq!(apply_transformation("x", "ixt-sec:numinf").unwrap(), "INF");
        assert_eq!(
            apply_transformation("x", "ixt-sec:numneginf").unwrap(),
            "-INF"
        );
        assert_eq!(apply_transformation("x", "ixt-sec:numnan").unwrap(), "NaN");
    }

    #[test]
    fn test_countrynameen() {
        // Major countries
        assert_eq!(countrynameen("United States").unwrap(), "US");
        assert_eq!(countrynameen("United States of America").unwrap(), "US");
        assert_eq!(countrynameen("U.S.A.").unwrap(), "US");
        assert_eq!(countrynameen("USA").unwrap(), "US");

        assert_eq!(countrynameen("United Kingdom").unwrap(), "GB");
        assert_eq!(countrynameen("U.K.").unwrap(), "GB");
        assert_eq!(countrynameen("Britain").unwrap(), "GB");
        assert_eq!(countrynameen("Great Britain").unwrap(), "GB");
        assert_eq!(countrynameen("England").unwrap(), "GB");

        assert_eq!(countrynameen("China").unwrap(), "CN");
        assert_eq!(countrynameen("People's Republic of China").unwrap(), "CN");

        assert_eq!(countrynameen("Canada").unwrap(), "CA");
        assert_eq!(countrynameen("Germany").unwrap(), "DE");
        assert_eq!(countrynameen("France").unwrap(), "FR");
        assert_eq!(countrynameen("Japan").unwrap(), "JP");
        assert_eq!(countrynameen("Australia").unwrap(), "AU");
        assert_eq!(countrynameen("Brazil").unwrap(), "BR");
        assert_eq!(countrynameen("India").unwrap(), "IN");

        // Countries with special characters
        assert_eq!(countrynameen("Côte d'Ivoire").unwrap(), "CI");
        assert_eq!(countrynameen("Curaçao").unwrap(), "CW");
        assert_eq!(countrynameen("Réunion").unwrap(), "RE");

        // Countries with long official names
        assert_eq!(countrynameen("The Republic of Argentina").unwrap(), "AR");
        assert_eq!(countrynameen("Argentina").unwrap(), "AR");
        assert_eq!(
            countrynameen("The Federative Republic of Brazil").unwrap(),
            "BR"
        );

        // Test via registry
        assert_eq!(
            apply_transformation("United States", "ixt-sec:countrynameen").unwrap(),
            "US"
        );

        // Test invalid country
        assert!(countrynameen("Atlantis").is_err());
    }
}
