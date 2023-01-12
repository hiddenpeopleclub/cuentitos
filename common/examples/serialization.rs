use cuentitos_common::*;
use std::fs::File;
use std::io::Write;

use rmp_serde::Serializer;
use serde::ser::Serialize;

fn main() {
  let element = EventBuilder::new()
    .id("my-event")
    .unique()
    .title("My Event")
    .description("My event description")
    .choice(EventChoice::default())
    .require(EventRequirement::default())
    .build();
  let mut result: Vec<u8> = Vec::new();
  element
    .serialize(&mut Serializer::new(&mut result))
    .unwrap();
  let mut file = File::create("default.mp").unwrap();
  file.write_all(&result).unwrap();
}
