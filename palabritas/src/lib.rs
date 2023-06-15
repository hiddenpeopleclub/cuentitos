mod parser;
pub use parser::parse_file;
pub use parser::parse_file_from_path;
#[macro_use]
extern crate pest_derive;

mod error;
