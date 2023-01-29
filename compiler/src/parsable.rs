use cuentitos_common::Config;

pub trait Parsable<T> {
  fn parse<S>(_content: S, _config: &Config) -> Result<T, String>
  where
    S: AsRef<str>,
  {
    todo!("Needs implementation")
  }
}
