//! FIGlet font file (.flf) and TheLetterFont (.tlf) parser.

use std::collections::HashMap;

use crate::font::Font;

/// Supported font format signatures.
const FLF_SIGNATURE: &str = "flf2a";
const TLF_SIGNATURE: &str = "tlf2a";

/// Parse error types.
#[derive(Debug)]
pub enum ParseError {
    InvalidHeader(String),
    InvalidCharacter(String),
    UnexpectedEndOfFile,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidHeader(msg) => write!(f, "Invalid header: {msg}"),
            ParseError::InvalidCharacter(msg) => write!(f, "Invalid character: {msg}"),
            ParseError::UnexpectedEndOfFile => write!(f, "Unexpected end of file"),
        }
    }
}

impl std::error::Error for ParseError {}

/// FLF file header information.
#[derive(Debug)]
struct FlfHeader {
    hardblank: char,
    height: usize,
    _baseline: usize,
    _max_length: usize,
    _old_layout: i32,
    comment_lines: usize,
}

/// Parse an FLF font file from string content.
pub fn parse_flf(name: &str, content: &str) -> Result<Font, ParseError> {
    let mut lines = content.lines();

    // Parse header
    let header_line = lines.next().ok_or(ParseError::UnexpectedEndOfFile)?;
    let header = parse_header(header_line)?;

    // Skip comment lines
    for _ in 0..header.comment_lines {
        lines.next();
    }

    // Parse characters
    let mut chars: HashMap<char, Vec<String>> = HashMap::new();

    // Standard ASCII characters start at 32 (space) and go to 126 (~)
    for ascii_code in 32u8..=126 {
        let char_lines = parse_character(&mut lines, header.height, header.hardblank)?;
        chars.insert(ascii_code as char, char_lines);
    }

    Ok(Font {
        name: name.to_string(),
        height: header.height,
        chars,
    })
}

/// Parse the FLF/TLF header line.
fn parse_header(line: &str) -> Result<FlfHeader, ParseError> {
    // Format: flf2a[hardblank] height baseline max_length old_layout comment_lines ...
    // Or:     tlf2a[hardblank] height baseline max_length old_layout comment_lines ...
    let signature_len = if line.starts_with(FLF_SIGNATURE) {
        FLF_SIGNATURE.len()
    } else if line.starts_with(TLF_SIGNATURE) {
        TLF_SIGNATURE.len()
    } else {
        return Err(ParseError::InvalidHeader(
            "Missing flf2a or tlf2a signature".to_string(),
        ));
    };

    let hardblank = line
        .chars()
        .nth(signature_len)
        .ok_or_else(|| ParseError::InvalidHeader("Missing hardblank character".to_string()))?;

    let parts: Vec<&str> = line[signature_len + 1..].split_whitespace().collect();
    if parts.len() < 5 {
        return Err(ParseError::InvalidHeader(
            "Not enough header fields".to_string(),
        ));
    }

    let height = parts[0]
        .parse()
        .map_err(|_| ParseError::InvalidHeader("Invalid height".to_string()))?;
    let baseline = parts[1]
        .parse()
        .map_err(|_| ParseError::InvalidHeader("Invalid baseline".to_string()))?;
    let max_length = parts[2]
        .parse()
        .map_err(|_| ParseError::InvalidHeader("Invalid max_length".to_string()))?;
    let old_layout = parts[3]
        .parse()
        .map_err(|_| ParseError::InvalidHeader("Invalid old_layout".to_string()))?;
    let comment_lines = parts[4]
        .parse()
        .map_err(|_| ParseError::InvalidHeader("Invalid comment_lines".to_string()))?;

    Ok(FlfHeader {
        hardblank,
        height,
        _baseline: baseline,
        _max_length: max_length,
        _old_layout: old_layout,
        comment_lines,
    })
}

/// Parse a single character from the FLF file.
fn parse_character<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
    height: usize,
    hardblank: char,
) -> Result<Vec<String>, ParseError> {
    let mut char_lines = Vec::with_capacity(height);

    for i in 0..height {
        let line = lines.next().ok_or(ParseError::UnexpectedEndOfFile)?;

        // Remove end markers (@ or @@)
        // TLF format may have trailing whitespace after @ markers, so trim whitespace first
        let cleaned = if i == height - 1 {
            // Last line ends with @@
            line.trim_end().trim_end_matches('@')
        } else {
            // Other lines end with @
            line.trim_end().trim_end_matches('@')
        };

        // Replace hardblank with space
        let final_line = cleaned.replace(hardblank, " ");

        char_lines.push(final_line);
    }

    Ok(char_lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flf_header() {
        let header = parse_header("flf2a$ 5 4 13 15 10 0 22415").unwrap();
        assert_eq!(header.hardblank, '$');
        assert_eq!(header.height, 5);
        assert_eq!(header.comment_lines, 10);
    }

    #[test]
    fn test_parse_tlf_header() {
        let header = parse_header("tlf2a$ 8 7 16 -1 4 0 0 0").unwrap();
        assert_eq!(header.hardblank, '$');
        assert_eq!(header.height, 8);
        assert_eq!(header.comment_lines, 4);
    }
}
