use cuentitos_common::*;
use std::io::Write;
use std::fs::File;

use serde::ser::Serialize;
use rmp_serde::Serializer;

fn main()
{
  let element = EventBuilder::new()
      .id("my-event")
      .unique()
      .title("My Event")
      .description("My event description")
      .choice(EventChoice::default())
      .require(EventRequirement::default())
      .build();
  let mut result: Vec<u8> = Vec::new();
  element.serialize(&mut Serializer::new(&mut result)).unwrap();
  let mut file = File::create("default.mp").unwrap();
  file.write_all(&result).unwrap();
}
