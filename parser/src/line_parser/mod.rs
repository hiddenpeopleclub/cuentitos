use crate::ParseError;

pub struct Line<'a> {
    pub text: &'a str,
}

pub struct ParseResult {
    pub string: String,
}

pub fn parse(line: Line) -> Result<ParseResult, ParseError> {
    Ok(ParseResult {
        string: line.text.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let line = Line {
            text: "Hello, world!",
        };

        let result = super::parse(line).unwrap();
        assert_eq!(result.string, "Hello, world!");
    }
}
