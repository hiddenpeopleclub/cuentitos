extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::{Parser, iterators::Pair};

#[derive(Parser)]
#[grammar = "palabritas.pest"]
pub struct PalabritasParser;

fn main() {

  let unparsed_file = std::fs::read_to_string("example.palabritas").expect("cannot read file");
  let file = PalabritasParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next().unwrap();

  for line in file.into_inner() {
        match line.as_rule() {
            Rule::text_shown => {
              debug_print(line);
            }
            Rule::option => {
              debug_print(line);
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
}


fn debug_print(pair: Pair<Rule>)
{
  let text_parts = pair.into_inner(); 
  let mut indent_count = 0;
  for text_part in text_parts
  {
    if text_part.as_rule() == Rule::indent
    {
      indent_count+=1;
    } else
    if text_part.as_rule() == Rule::one_off
    {
      println!("one_off: {}",text_part.as_str() );
    } else
    if text_part.as_rule() == Rule::line
    { 
      let mut line_parts = text_part.into_inner();
      println!("(Indent{}){}",indent_count, line_parts.next().unwrap().as_str());
      for requirement_or_frequency in line_parts
      {
        if requirement_or_frequency.as_rule() == Rule::requirement
        {
          let mut requirement_parts = requirement_or_frequency.into_inner();
          requirement_parts.next();
          let mut req_str = "req: ".to_string();
          while let Some(requirement_part) = requirement_parts.next()
          {
            req_str = req_str + requirement_part.as_str();
          }

          println!("{}",req_str);
          
        } else
        if requirement_or_frequency.as_rule() == Rule::frequency
        {
          let mut frequency_parts = requirement_or_frequency.into_inner();
          frequency_parts.next();
          let mut freq_str = "freq: ".to_string();
          while let Some(frequency_part) = frequency_parts.next()
          {
            freq_str = freq_str + frequency_part.as_str();
          }

          println!("{}",freq_str);
        }
      }
    }
  }
}