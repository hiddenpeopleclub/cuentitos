use std::path::Path;
use std::path::PathBuf;

#[derive(Clone)]
pub struct TestCase {
    pub name: String,
    pub script: String,
    pub input: String,
    pub result: String,
    pub path: PathBuf,
    pub disabled: bool,
}

fn parse_name(content: &str) -> String {
    content
        .lines()
        .next()
        .unwrap()
        .split("# ")
        .collect::<Vec<&str>>()[1]
        .to_string()
}

fn parse_markdown_blog(content: &str, language: &str) -> String {
    content
        .split(&format!("```{}\n", language))
        .collect::<Vec<&str>>()[1]
        .split("```")
        .collect::<Vec<&str>>()[0]
        .trim()
        .to_string()
}

impl TestCase {
    pub fn from_string<A, B>(content: A, path: B) -> Self
    where
        A: AsRef<str>,
        B: AsRef<Path>,
    {
        let name = parse_name(content.as_ref());
        let script = parse_markdown_blog(content.as_ref(), "cuentitos");
        let input = parse_markdown_blog(content.as_ref(), "input");
        let result = parse_markdown_blog(content.as_ref(), "result");
        let disabled = content.as_ref().trim().ends_with("!!! disabled");

        TestCase {
            name,
            script,
            input,
            result,
            path: path.as_ref().into(),
            disabled,
        }
    }
}

mod test {
    #[test]
    fn test_case_from_string_with_simple_test() {
        let content = include_str!("../../compatibility-tests/00000000001-single-line-and-end.md");

        let test_case = super::TestCase::from_string(
            content,
            "compatibility-tests/00000000001-single-line-and-end.md",
        );

        assert_eq!(test_case.name, "Single Line and End");
        assert_eq!(test_case.script, "This is a single line");
        assert_eq!(test_case.input, "n\nn");
        assert_eq!(test_case.result, "START\nThis is a single line\nEND");
        assert_eq!(test_case.disabled, false);
    }

    #[test]
    fn test_case_from_string_with_disabled_test() {
        let content = "# Test Name\n\nTest description\n\n## Script\n```cuentitos\ntest\n```\n\n## Input\n```input\nn\n```\n\n## Result\n```result\ntest\n```\n\n!!! disabled";

        let test_case = super::TestCase::from_string(content, "test.md");

        assert_eq!(test_case.disabled, true);
    }
}
