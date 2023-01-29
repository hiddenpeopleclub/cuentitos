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
  if buffer.is_null() { return 0; }

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

#[no_mangle]
pub extern "C" fn cuentitos_set_seed(id: RuntimeId, seed: u64) {
  if invalid_runtime(id) {
    return;
  }
  get_runtime(id).set_seed(seed);
}

#[no_mangle]
pub extern "C" fn cuentitos_set_int_resource(id: RuntimeId, resource: Cstring, value: i32) -> bool {
  set_resource(id, resource, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_set_float_resource(
  id: RuntimeId,
  resource: Cstring,
  value: f32,
) -> bool {
  set_resource(id, resource, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_set_bool_resource(
  id: RuntimeId,
  resource: Cstring,
  value: bool,
) -> bool {
  set_resource(id, resource, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_get_int_resource(
  id: RuntimeId,
  resource: Cstring,
  value: *mut i32,
) -> bool {
  get_resource(id, resource, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_get_float_resource(
  id: RuntimeId,
  resource: Cstring,
  value: *mut f32,
) -> bool {
  get_resource(id, resource, value)
}

#[no_mangle]
pub extern "C" fn cuentitos_get_bool_resource(
  id: RuntimeId,
  resource: Cstring,
  value: *mut bool,
) -> bool {
  get_resource(id, resource, value)
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
pub extern "C" fn cuentitos_set_tile(id: RuntimeId, tile: Cstring) -> bool {
  if invalid_runtime(id) {
    return false;
  }
  if tile.is_null() {
    return false;
  }
  let tile = rust_string(tile);
  Ok(()) == get_runtime(id).set_tile(tile)
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
    serialize_to_buffer(event, buffer, length);
  } else {
    zero(length)
  }
}

#[no_mangle]
pub extern "C" fn cuentitos_set_choice(id: usize, choice_id: usize) -> bool {
  if invalid_runtime(id) {
    return false;
  }
  Ok(()) == get_runtime(id).set_choice(choice_id)
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
    serialize_to_buffer(modifiers, buffer, length);
    return true;
  }

  false
}

fn set_resource<T>(id: RuntimeId, resource: Cstring, value: T) -> bool
where
  T: Display,
{
  if invalid_runtime(id) {
    return false;
  }
  if resource.is_null() {
    return false;
  }
  let resource = rust_string(resource);
  Ok(()) == get_runtime(id).set_resource(resource, value)
}

fn get_resource<T>(id: RuntimeId, resource: Cstring, value: *mut T) -> bool
where
  T: Display + std::str::FromStr + std::default::Default,
{
  if invalid_runtime(id) {
    return false;
  }
  if resource.is_null() {
    return false;
  }
  if value.is_null() {
    return false;
  }

  let runtime = get_runtime(id);
  let resource = rust_string(resource);
  if let Ok(resource_value) = runtime.get_resource(resource) {
    unsafe {
      *value = resource_value;
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

fn serialize_to_buffer<U>(element: U, buffer: *mut u8, length: *mut usize)
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
