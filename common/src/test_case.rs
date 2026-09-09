use std::path::Path;
use std::path::PathBuf;

/// One locale's translation file as it appears in a `## Translations` or
/// `## Expected Translations` section. The fence language names the locale.
#[derive(Clone, Debug, PartialEq)]
pub struct TranslationFile {
    pub locale: String,
    pub contents: String,
}

#[derive(Clone)]
pub struct TestCase {
    pub name: String,
    pub script: String,
    pub input: String,
    pub result: String,
    pub path: PathBuf,
    pub pending_reason: Option<String>,
    /// Translation files to materialize under `locales/` before the run.
    pub translations: Vec<TranslationFile>,
    /// Translation files the compiler must leave under `locales/` afterwards.
    pub expected_translations: Vec<TranslationFile>,
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

/// Collects every fenced block under each `## <heading>` section, pairing the
/// fence language with the block body. A heading may appear more than once, one
/// occurrence per locale.
fn parse_translation_sections(content: &str, heading: &str) -> Vec<TranslationFile> {
    let heading_line = format!("## {}", heading);
    let lines: Vec<&str> = content.lines().collect();
    let mut translation_files = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        if lines[index].trim() != heading_line {
            index += 1;
            continue;
        }
        index += 1;

        while index < lines.len() && !lines[index].starts_with("## ") {
            let locale = lines[index]
                .strip_prefix("```")
                .map(str::trim)
                .unwrap_or("");
            index += 1;
            if locale.is_empty() {
                continue;
            }

            let body_start = index;
            while index < lines.len() && !lines[index].starts_with("```") {
                index += 1;
            }
            let contents = lines[body_start..index].join("\n").trim_end().to_string();
            translation_files.push(TranslationFile {
                locale: locale.to_string(),
                contents,
            });
            index += 1;
        }
    }

    translation_files
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
        let translations = parse_translation_sections(content.as_ref(), "Translations");
        let expected_translations =
            parse_translation_sections(content.as_ref(), "Expected Translations");

        TestCase {
            name,
            script,
            input,
            result,
            path: path.as_ref().into(),
            pending_reason,
            translations,
            expected_translations,
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
    fn test_case_from_string_has_no_translations_by_default() {
        let content =
            include_str!("../../compatibility-tests/strings/feature/single-line-and-end.md");

        let test_case = super::TestCase::from_string(content, "single-line-and-end.md");

        assert!(test_case.translations.is_empty());
        assert!(test_case.expected_translations.is_empty());
    }

    #[test]
    fn test_case_from_string_reads_translation_sections() {
        let content = concat!(
            "# Translated Test\n\n",
            "## Script\n```cuentitos\nHello.\n```\n\n",
            "## Translations\n```es\nid,line,original,translation,status\n",
            "2d8bd7d9,1,Hello.,Hola.,\n```\n\n",
            "## Input\n```input\ns\n```\n\n",
            "## Result\n```result\nSTART\nHello.\nEND\n```\n\n",
            "## Expected Translations\n```es\nid,line,original,translation,status\n",
            "2d8bd7d9,1,Hello.,Hola.,\n```\n",
        );

        let test_case = super::TestCase::from_string(content, "translated-test.md");

        assert_eq!(test_case.script, "Hello.");
        assert_eq!(test_case.result, "START\nHello.\nEND");
        assert_eq!(test_case.translations.len(), 1);
        assert_eq!(test_case.translations[0].locale, "es");
        assert_eq!(
            test_case.translations[0].contents,
            "id,line,original,translation,status\n2d8bd7d9,1,Hello.,Hola.,"
        );
        assert_eq!(
            test_case.expected_translations, test_case.translations,
            "the expected section is parsed the same way as the pre-existing one"
        );
    }

    #[test]
    fn test_case_from_string_reads_one_section_per_locale() {
        let content = concat!(
            "# Two Locales\n\n",
            "## Script\n```cuentitos\nHello.\n```\n\n",
            "## Expected Translations\n```es\nid\nes-row\n```\n\n",
            "## Expected Translations\n```fr\nid\nfr-row\n```\n\n",
            "## Input\n```input\ns\n```\n\n",
            "## Result\n```result\nSTART\n```\n",
        );

        let test_case = super::TestCase::from_string(content, "two-locales.md");

        let locales: Vec<&str> = test_case
            .expected_translations
            .iter()
            .map(|translation| translation.locale.as_str())
            .collect();
        assert_eq!(locales, vec!["es", "fr"]);
        assert_eq!(test_case.expected_translations[1].contents, "id\nfr-row");
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
