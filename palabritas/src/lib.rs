mod parser;
pub use parser::parse_database;
pub use parser::parse_database_from_path;
#[macro_use]
extern crate pest_derive;

mod error;
