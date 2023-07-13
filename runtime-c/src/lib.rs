use core::fmt::Display;
use cuentitos_common::Database;
use cuentitos_runtime::Runtime;
use rmp_serde::Serializer;
use serde::Serialize;
use std::ffi::CStr;
use std::slice;

type Cstring = *const libc::c_char;
type RuntimeId = usize;
type DatabaseId = usize;

#[repr(C)]
pub enum TimeOfDay {
  Morning,
  Noon,
  Evening,
  Night,
}

static mut DATABASES: Vec<Database> = vec![];
static mut RUNTIMES: Vec<Runtime> = vec![];

#[no_mangle]
pub extern "C" fn cuentitos_load_db(buffer: *const u8, length: usize) -> DatabaseId {
  if buffer.is_null() {
    return 0;
  }

  let rust_buffer: &[u8] = unsafe { slice::from_raw_parts(buffer, length as usize) };
  let db = Database::from_u8(rust_buffer);

  match db {
    Ok(db) => unsafe {
      let id = DATABASES.len();
      DATABASES.push(db);
      id
    },
    Err(err) => {
      println!("Error Loading DB: {}", err);
      0
    }
  }
}

#[no_mangle]
pub extern "C" fn cuentitos_new_runtime(id: DatabaseId) -> RuntimeId {
  unsafe {
    if DATABASES.len() > id {
      let runtime = Runtime::new(DATABASES[id].clone());
      let id = RUNTIMES.len();
      RUNTIMES.push(runtime);
      id
    } else {
      panic!("Database {} does not exist.", id)
    }
  }
}

pub extern "C" fn cuentitos_set_locale(id: RuntimeId, locale: Cstring) -> bool {
  if invalid_runtime(id) {
    return false;
  }

  get_runtime(id).set_locale(rust_string(locale)) == Ok(())
}

#[no_mangle]
pub extern "C" fn cuentitos_set_seed(id: RuntimeId, seed: u64) {
  if invalid_runtime(id) {
    return;
  }
  get_runtime(id).set_seed(seed);
}

#[no_mangle]
pub extern "C" fn cuentitos_set_int_variable(id: RuntimeId, variable: Cstring, value: i32) -> bool {
  set_variable(id, variable, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_set_float_variable(
  id: RuntimeId,
  variable: Cstring,
  value: f32,
) -> bool {
  set_variable(id, variable, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_set_bool_variable(
  id: RuntimeId,
  variable: Cstring,
  value: bool,
) -> bool {
  set_variable(id, variable, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_get_int_variable(
  id: RuntimeId,
  variable: Cstring,
  value: *mut i32,
) -> bool {
  get_variable(id, variable, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_get_float_variable(
  id: RuntimeId,
  variable: Cstring,
  value: *mut f32,
) -> bool {
  get_variable(id, variable, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_get_bool_variable(
  id: RuntimeId,
  variable: Cstring,
  value: *mut bool,
) -> bool {
  get_variable(id, variable, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_set_item(id: RuntimeId, item: Cstring, value: u8) -> bool {
  if invalid_runtime(id) {
    return false;
  }
  if item.is_null() {
    return false;
  }
  let item = rust_string(item);
  Ok(()) == get_runtime(id).set_item(item, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_set_time_of_day(id: RuntimeId, time_of_day: TimeOfDay) -> bool {
  if invalid_runtime(id) {
    return false;
  }

  match time_of_day {
    TimeOfDay::Morning => get_runtime(id).set_time_of_day(cuentitos_common::TimeOfDay::Morning),
    TimeOfDay::Noon => get_runtime(id).set_time_of_day(cuentitos_common::TimeOfDay::Noon),
    TimeOfDay::Evening => get_runtime(id).set_time_of_day(cuentitos_common::TimeOfDay::Evening),
    TimeOfDay::Night => get_runtime(id).set_time_of_day(cuentitos_common::TimeOfDay::Night),
  }
  true
}

#[no_mangle]
pub extern "C" fn cuentitos_get_reputation(
  id: RuntimeId,
  reputation: Cstring,
  value: *mut i32,
) -> bool {
  if invalid_runtime(id) {
    return false;
  }
  if reputation.is_null() {
    return false;
  }
  if value.is_null() {
    return false;
  }

  let reputation = rust_string(reputation);
  if let Ok(reputation_value) = get_runtime(id).get_reputation(reputation) {
    unsafe { *value = reputation_value }
    return true;
  }
  false
}

#[no_mangle]
pub extern "C" fn cuentitos_get_decision(
  id: RuntimeId,
  decision: Cstring,
  value: *mut bool,
) -> bool {
  if invalid_runtime(id) {
    return false;
  }
  if decision.is_null() {
    return false;
  }
  if value.is_null() {
    return false;
  }

  let decision = rust_string(decision);
  unsafe { *value = get_runtime(id).decision_taken(decision) }
  true
}

#[no_mangle]
pub extern "C" fn cuentitos_get_items(id: usize, buffer: *mut u8, length: *mut usize) {
  if buffer.is_null() {
    return;
  }
  if length.is_null() {
    return;
  }
  if invalid_runtime(id) {
    return;
  }

  serialize_to_buffer(&get_runtime(id).database.items, buffer, length);
}

#[no_mangle]
pub extern "C" fn cuentitos_next_event(id: usize, buffer: *mut u8, length: *mut usize) {
  if buffer.is_null() {
    return;
  }
  if length.is_null() {
    return;
  }
  if invalid_runtime(id) {
    return;
  }

  if let Some(event) = get_runtime(id).next_event() {
    serialize_to_buffer(&event, buffer, length);
  } else {
    zero(length)
  }
}

#[no_mangle]
pub extern "C" fn cuentitos_set_choice(
  id: usize,
  choice_id: usize,
  buffer: *mut u8,
  length: *mut usize,
) -> bool {
  if buffer.is_null() {
    return false;
  }
  if length.is_null() {
    return false;
  }
  if invalid_runtime(id) {
    return false;
  }

  match get_runtime(id).set_choice(choice_id) {
    Ok(modifiers) => {
      serialize_to_buffer(&modifiers, buffer, length);
      true
    }
    Err(_) => false,
  }
}

#[no_mangle]
pub extern "C" fn cuentitos_current_modifiers(
  id: usize,
  buffer: *mut u8,
  length: *mut usize,
) -> bool {
  if buffer.is_null() {
    return false;
  }
  if length.is_null() {
    return false;
  }
  if invalid_runtime(id) {
    return false;
  }

  if let Some(modifiers) = get_runtime(id).current_modifiers() {
    serialize_to_buffer(&modifiers, buffer, length);
    return true;
  }

  false
}

fn set_variable<T>(id: RuntimeId, variable: Cstring, value: T) -> bool
where
  T: Display,
{
  if invalid_runtime(id) {
    return false;
  }
  if variable.is_null() {
    return false;
  }
  let variable = rust_string(variable);
  Ok(()) == get_runtime(id).set_variable(variable, value)
}

fn get_variable<T>(id: RuntimeId, variable: Cstring, value: *mut T) -> bool
where
  T: Display + std::str::FromStr + std::default::Default,
{
  if invalid_runtime(id) {
    return false;
  }
  if variable.is_null() {
    return false;
  }
  if value.is_null() {
    return false;
  }

  let runtime = get_runtime(id);
  let variable = rust_string(variable);
  if let Ok(variable_value) = runtime.get_variable(variable) {
    unsafe {
      *value = variable_value;
      return true;
    }
  }
  false
}

fn invalid_runtime(id: usize) -> bool {
  unsafe { id >= RUNTIMES.len() }
}

fn get_runtime(id: usize) -> &'static mut Runtime {
  unsafe { &mut RUNTIMES[id] }
}

fn rust_string(string: Cstring) -> String {
  unsafe { CStr::from_ptr(string) }
    .to_str()
    .unwrap_or_default()
    .to_string()
}

fn serialize<U>(element: U) -> cuentitos_common::Result<Vec<u8>>
where
  U: Serialize,
{
  let mut result: Vec<u8> = Vec::new();
  element.serialize(&mut Serializer::new(&mut result))?;
  Ok(result)
}

fn serialize_to_buffer<U>(element: &U, buffer: *mut u8, length: *mut usize)
where
  U: Serialize,
{
  if buffer.is_null() {
    return;
  }
  if length.is_null() {
    return;
  }
  let serialized = serialize(element).unwrap_or_default();
  unsafe {
    *length = serialized.len();
    std::ptr::copy_nonoverlapping(serialized.as_ptr(), buffer, serialized.len());
  }
}

fn zero(length: *mut usize) {
  if length.is_null() {
    return;
  }
  unsafe {
    *length = 0;
  }
}
