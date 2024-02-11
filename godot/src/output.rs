#[derive(GodotClass)]
#[class(init, base=Object)]
struct Block {
  #[var]
  kind: GString,
  #[var]
  id: GString,
  #[var]
  name: GString,
  #[var]
  next: Gd<NextBlock>,
  #[var]
  settings: Gd<BlockSettings>,
  base: Base<Object>
}

impl Block {
  fn new_gd(block: cuentitos_runtime::Block) -> Gd<Self> {
    let mut kind = "None";
    let mut id = "";
    let mut name = "";

    match block {
        cuentitos_runtime::Block::Text { text, settings } => todo!(),
        cuentitos_runtime::Block::Choice { text, settings } => todo!(),
        cuentitos_runtime::Block::Bucket { name, settings } => todo!(),
        cuentitos_runtime::Block::Section { settings } => todo!(),
        cuentitos_runtime::Block::Divert { next, settings } => todo!(),
        cuentitos_runtime::Block::BoomerangDivert { next, settings } => todo!(),
    }
    Gd::from_init_fn(|base| {
      Self {
        text,
        choices,
        blocks,
        base,
      }
    })
}
#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct NextBlock {
  #[var]
  kind: GString,
  #[var]
  id: GString,
  base: Base<RefCounted>
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct BlockSettings {
  #[var]
  children: Array<GString>,
  #[var]
  chance: Gd<Chance>,
  #[var]
  frequency_modifiers: Array<Gd<FrequencyModifier>>,
  #[var]
  requirements: Array<Gd<Requirement>>,
  #[var]
  modifiers: Array<Gd<Modifier>>,
  #[var]
  unique: bool,
  #[var]
  tags: Array<GString>,
  #[var]
  functions: Array<Gd<Function>>,
  #[var]
  script: Gd<Script>,
  #[var]
  section: GString,
  base: Base<RefCounted>
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct Script {
  #[var]  
  file: GString,
  #[var]
  line: u32,
  #[var]
  col: u32,
  base: Base<RefCounted>
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct Chance {
  #[var]
  kind: GString,
  #[var]
  value: f32,
  base: Base<RefCounted>
}

#[derive(GodotClass)]
#[class(init, base=Object)]
struct FrequencyModifier {
  #[var]
  condition: Gd<Condition>,
  #[var]
  value: i32,
  base: Base<Object>
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct Condition {
  #[var]
  variable: GString,
  #[var]
  operator: GString,
  #[var]
  value: GString,
  base: Base<RefCounted>
}

#[derive(GodotClass)]
#[class(init, base=Object)]
struct Function {
  #[var]
  name: GString,
  #[var]
  parameters: Array<GString>,
  base: Base<Object>
}

#[derive(GodotClass)]
#[class(init, base=Object)]
struct Requirement {
  #[var]
  condition: Gd<Condition>,
  base: Base<Object>
}

#[derive(GodotClass)]
#[class(init, base=Object)]
struct Modifier {
  #[var]
  variable: GString,
  #[var]
  value: GString,
  #[var]
  operator: GString,
  base: Base<Object>
}