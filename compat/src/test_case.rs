pub struct TestCase {
  name: String,
  script: String,
  input: String,
  result: String
}


fn parse_name(content: &str) -> String {
  content.lines().next().unwrap().split("# ").collect::<Vec<&str>>()[1].to_string()
}

fn parse_markdown_blog(content: &str, language: &str) -> String {
  content.split(&format!("```{}\n", language)).collect::<Vec<&str>>()[1]
    .split("```").collect::<Vec<&str>>()[0].trim().to_string()
}


impl TestCase {
  pub fn from_string<A>(content: A) -> Self
    where A : AsRef<str>
  {
    let name = parse_name(content.as_ref());
    let script = parse_markdown_blog(content.as_ref(), "cuentitos");
    let input = parse_markdown_blog(content.as_ref(), "input");
    let result = parse_markdown_blog(content.as_ref(), "result");

    TestCase { name, script, input, result }
  }
}

mod test {
  use super::*;

  #[test]
  fn test_case_from_string_with_simple_test() {
    let content = include_str!(
      "../../compatibility-tests/00000000001-single-line-and-end.md"
    );

    let test_case = TestCase::from_string(content);
    assert_eq!(test_case.name, "Single Line and End");
    assert_eq!(test_case.script, "This is a single line");
    assert_eq!(test_case.input, "n");
    assert_eq!(test_case.result,
"This is a single line
END"
);
  }

}
