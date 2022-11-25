use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::Read;

use rmp_serde::encode::*;
use crate::Result;

pub fn load_mp_fixture<U>(fixture: U) -> Result<Vec<u8>> 
where
  U: AsRef<str>,
{
  let path = format!("fixtures/{}.mp", fixture.as_ref());
  let mut f = File::open(&path).expect("no file found");
  let metadata = fs::metadata(&path).expect("unable to read metadata");
  let mut buffer = vec![0; metadata.len() as usize];
  f.read(&mut buffer).expect("buffer overflow");

  Ok(buffer)
}

pub fn serialize<U>(element: U) -> Result<Vec<u8>>
where
  U: Serialize,
{
  let mut result: Vec<u8> = Vec::new();
  element.serialize(&mut Serializer::new(&mut result))?;
  Ok(result)
}
