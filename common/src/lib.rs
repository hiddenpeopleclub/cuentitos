pub mod test_case;

pub type StringId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    String(StringId)
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Database {
    pub blocks: Vec<Block>,
    pub strings: Vec<String>
}
