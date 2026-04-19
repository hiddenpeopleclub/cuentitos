use std::path::Path;
use std::path::PathBuf;

#[derive(Clone)]
pub struct TestCase {
    pub name: String,
    pub script: String,
    pub input: String,
    pub result: String,
    pub path: PathBuf,
    pub pending_reason: Option<String>,
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

fn parse_markdown_block(content: &str, language: &str) -> String {
    let block = content
        .split(&format!("```{}\n", language))
        .collect::<Vec<&str>>()[1]
        .split("```")
        .collect::<Vec<&str>>()[0];

    // Trim only trailing whitespace, preserve leading whitespace on each line
    block.trim_end().to_string()
}

fn parse_pending_reason(content: &str) -> Option<String> {
    let mut lines = content.lines().skip_while(|l| l.trim() != "## Pending");
    lines.next()?;
    let reason: Vec<&str> = lines.take_while(|l| !l.starts_with("## ")).collect();
    Some(reason.join("\n").trim().to_string())
}

impl TestCase {
    pub fn from_string<A, B>(content: A, path: B) -> Self
    where
        A: AsRef<str>,
        B: AsRef<Path>,
    {
        let name = parse_name(content.as_ref());
        let script = parse_markdown_block(content.as_ref(), "cuentitos");
        let input = parse_markdown_block(content.as_ref(), "input");
        let result = parse_markdown_block(content.as_ref(), "result");
        let pending_reason = parse_pending_reason(content.as_ref());

        TestCase {
            name,
            script,
            input,
            result,
            path: path.as_ref().into(),
            pending_reason,
        }
    }
}

mod test {
    #[test]
    fn test_case_from_string_with_simple_test() {
        let content =
            include_str!("../../compatibility-tests/strings/feature/single-line-and-end.md");

        let test_case = super::TestCase::from_string(
            content,
            "compatibility-tests/strings/feature/single-line-and-end.md",
        );

        assert_eq!(test_case.name, "Single Line and End");
        assert_eq!(test_case.script, "This is a single line");
        assert_eq!(test_case.input, "n\nn");
        assert_eq!(test_case.result, "START\nThis is a single line\nEND");
        assert!(test_case.pending_reason.is_none());
    }

    #[test]
    fn test_case_from_string_marks_pending() {
        let content = "# Pending Test\n\nDescription.\n\n## Pending\n\nFeature not implemented yet.\n\n## Script\n```cuentitos\nfoo\n```\n\n## Input\n```input\nn\n```\n\n## Result\n```result\nbar\n```\n";

        let test_case = super::TestCase::from_string(content, "pending-test.md");

        assert_eq!(
            test_case.pending_reason.as_deref(),
            Some("Feature not implemented yet.")
        );
    }
}
