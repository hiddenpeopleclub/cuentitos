use std::io::Result;

pub fn generate() -> Result<()> {

  let content = include_str!("../templates/index.html");

  crate::util::save("index.html", content)?;

  println!("- generated index.html");
  
  Ok(())
}