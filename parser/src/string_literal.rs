//! Shared double-quoted string-literal scanner.
//!
//! Both string-variable *defaults* (`--- variables` blocks) and string `set`
//! RHS *literals* accept the exact same surface syntax: a single double-quoted
//! literal recognizing the `\"`, `\n`, and `\\` escapes. Keeping one scanner
//! here guarantees the two sites stay byte-for-byte consistent — an escape
//! rule fixed in one place is inherited by the other rather than drifting
//! against a hand-rolled copy.
//!
//! The scanner is intentionally narrow: it parses *one* literal that occupies
//! the whole input (modulo surrounding whitespace). Any non-whitespace text
//! after the closing quote is reported as [`StringLiteralError::TrailingCharacters`]
//! so callers can surface an attempted concatenation (`"a" + "b"`) or a stray
//! second token (`"a" "b"`) as their own malformed-expression diagnostic.

/// Failure modes of [`parse_string_literal`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringLiteralError {
    /// The literal opened a double quote but never closed it.
    Unterminated,
    /// A backslash was followed by a character other than `"`, `n`, or `\`.
    InvalidEscape { sequence: String },
    /// Non-whitespace text followed the closing quote (e.g. `"a" "b"`).
    TrailingCharacters,
}

/// Parse a double-quoted string literal that occupies the whole `input`.
///
/// The caller is expected to have trimmed `input` and verified its leading
/// `"` (in debug builds a missing leading quote trips an assertion).
/// Recognizes the `\"`, `\n`, and `\\` escapes; any other backslash sequence
/// is an [`InvalidEscape`](StringLiteralError::InvalidEscape). A missing
/// closing quote is [`Unterminated`](StringLiteralError::Unterminated), and
/// any non-whitespace after the closing quote is
/// [`TrailingCharacters`](StringLiteralError::TrailingCharacters).
pub fn parse_string_literal(input: &str) -> Result<String, StringLiteralError> {
    let mut chars = input.chars();
    let opening = chars.next();
    debug_assert_eq!(opening, Some('"'), "caller guarantees a leading quote");

    let value = scan_quoted_body(&mut chars)?;

    if chars.as_str().trim().is_empty() {
        Ok(value)
    } else {
        Err(StringLiteralError::TrailingCharacters)
    }
}

/// Scan the body of a double-quoted literal whose opening `"` has **already
/// been consumed** from `chars`, reading through the closing quote and leaving
/// `chars` positioned just past it. Decodes the `\"`, `\n`, and `\\` escapes.
///
/// Unlike [`parse_string_literal`], this does not require the literal to fill
/// the input — it stops at the closing quote and ignores whatever follows. The
/// `req` boolean tokenizer uses it to lex one string literal embedded in a
/// larger token stream (`req name = "Aria" and ...`); `parse_string_literal`
/// reuses it for the whole-input form so both honor identical escape rules.
pub fn scan_quoted_body<I: Iterator<Item = char>>(
    chars: &mut I,
) -> Result<String, StringLiteralError> {
    let mut value = String::new();
    loop {
        match chars.next() {
            None => return Err(StringLiteralError::Unterminated),
            Some('"') => break,
            Some('\\') => match chars.next() {
                Some('"') => value.push('"'),
                Some('n') => value.push('\n'),
                Some('\\') => value.push('\\'),
                Some(other) => {
                    return Err(StringLiteralError::InvalidEscape {
                        sequence: format!("\\{other}"),
                    });
                }
                None => return Err(StringLiteralError::Unterminated),
            },
            Some(other) => value.push(other),
        }
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_literal() {
        assert_eq!(parse_string_literal("\"Hello\""), Ok("Hello".to_string()));
    }

    #[test]
    fn parses_empty_literal() {
        assert_eq!(parse_string_literal("\"\""), Ok(String::new()));
    }

    #[test]
    fn unescapes_supported_sequences() {
        assert_eq!(
            parse_string_literal("\"x\\ny\\\\z\\\"w\""),
            Ok("x\ny\\z\"w".to_string())
        );
    }

    #[test]
    fn rejects_unterminated_literal() {
        assert_eq!(
            parse_string_literal("\"Brenn"),
            Err(StringLiteralError::Unterminated)
        );
    }

    #[test]
    fn rejects_invalid_escape() {
        assert_eq!(
            parse_string_literal("\"a\\qb\""),
            Err(StringLiteralError::InvalidEscape {
                sequence: "\\q".to_string()
            })
        );
    }

    #[test]
    fn rejects_trailing_characters() {
        assert_eq!(
            parse_string_literal("\"a\" \"b\""),
            Err(StringLiteralError::TrailingCharacters)
        );
        assert_eq!(
            parse_string_literal("\"Hello, \" + \"world\""),
            Err(StringLiteralError::TrailingCharacters)
        );
    }

    #[test]
    fn ignores_trailing_whitespace() {
        assert_eq!(parse_string_literal("\"Hi\"   "), Ok("Hi".to_string()));
    }
}
