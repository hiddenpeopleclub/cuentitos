use core::fmt::Display;
use std::ffi::CStr;
use rmp_serde::Serializer;
use serde::Serialize;
use cuentitos_runtime::Runtime;
use cuentitos_common::{Database, ResourceKind};
use std::slice;

type Cstring = *const libc::c_char;

static mut DATABASES: Vec<Database> = vec![]; 
static mut RUNTIMES: Vec<Runtime> = vec![];

#[no_mangle]
pub extern "C" fn cuentitos_load_db(buffer: *const u8, length: usize) -> usize {
  if buffer.is_null() { return 0; }

  let rust_buffer: &[u8] = unsafe { slice::from_raw_parts(buffer, length as usize) };
  let db = Database::from_u8(rust_buffer);

  match db {
    Ok(db) => {
      unsafe {
        let id = DATABASES.len();
        DATABASES.push(db);
        return id;
      }
    },
    Err(err) => {
      println!("Error Loading DB: {}", err.to_string());
      return 0
    }
  };
}

#[no_mangle]
pub extern "C" fn cuentitos_new_runtime(id: usize) -> usize {
  unsafe {
    if DATABASES.len() > id {
      let runtime = Runtime::new(DATABASES[id].clone());
      let id = RUNTIMES.len();
      RUNTIMES.push(runtime);
      return id;
    } else {
      panic!("Database {} does not exist.", id)
    }
  }
}

fn invalid_runtime(id: usize) -> bool {
  unsafe {
    id >= RUNTIMES.len()
  }
}

fn get_runtime(id: usize) -> &'static mut Runtime {
  unsafe {
    &mut RUNTIMES[id]
  }
}

#[no_mangle]
pub extern "C" fn cuentitos_set_seed(id: usize, seed: u64) {
  if invalid_runtime(id) { return; }
  get_runtime(id).set_seed(seed);
}

#[no_mangle]
pub extern "C" fn cuentitos_set_int_resource(id: usize, resource: Cstring, value: i32) -> bool {
  return cuentitos_set_resource(id, resource, value);
}

#[no_mangle]
pub extern "C" fn cuentitos_set_float_resource(id: usize, resource: Cstring, value: f32) -> bool {
  return cuentitos_set_resource(id, resource, value);
}

#[no_mangle]
pub extern "C" fn cuentitos_set_bool_resource(id: usize, resource: Cstring, value: bool) -> bool {
  return cuentitos_set_resource(id, resource, value);
}

fn cuentitos_set_resource<T>(id: usize, resource: Cstring, value: T) -> bool 
where
  T: Display
{
  if invalid_runtime(id) { return false; }
  if resource.is_null() { return false; }
  let resource = rust_string(resource);
  Ok(()) == get_runtime(id).set_resource(resource, value)
}

#[no_mangle]
pub extern  "C" fn cuentitos_get_int_resource(id: usize, resource: Cstring, value: *mut i32) -> bool 
{
  return cuentitos_get_resource(id, resource, value);
}

#[no_mangle]
pub extern  "C" fn cuentitos_get_float_resource(id: usize, resource: Cstring, value: *mut f32) -> bool 
{
  return cuentitos_get_resource(id, resource, value);
}

#[no_mangle]
pub extern  "C" fn cuentitos_get_bool_resource(id: usize, resource: Cstring, value: *mut bool) -> bool 
{
  return cuentitos_get_resource(id, resource, value);
}

fn cuentitos_get_resource<T>(id: usize, resource: Cstring, value: *mut T) -> bool 
where
  T: Display + std::str::FromStr + std::default::Default
{
  if invalid_runtime(id) { return false; }
  if resource.is_null() { return false; }
  if value.is_null() { return false; }

  let runtime = get_runtime(id);
  let resource = rust_string(resource);
  if let Ok(resource_value) = runtime.get_resource(resource) {
    unsafe { 
      *value = resource_value;
      return true;
    }    
  }
  

  // if let Some(resource_kind) = runtime.database.config.resources.get(&resource) {
  //   match resource_kind {
  //     ResourceKind::Integer => {
  //       if let Ok(value) = runtime.get_resource::<String, i32>(resource.clone()) {
  //         serialize_to_buffer(value, buffer, length);
  //         return true;
  //       }else{
  //         zero(length)
  //       }
  //     },
  //     ResourceKind::Float => {},
  //     ResourceKind::Bool => {},
  //   }
  // }
  return false;

}

// pub fn set_time_of_day(&mut self, time_of_day: TimeOfDay)
// pub fn get_reputation<T>(&self, reputation_id: T) -> Result<i32, String> 
// pub fn decision_taken<T>(&self, decision_id: T) -> bool


#[no_mangle]
pub extern "C" fn next_event(id: usize, buffer: *mut u8, length: *mut usize) {
  if buffer.is_null() { return; }
  if length.is_null() { return; }
  if invalid_runtime(id) { return; }

  if let Some(event) = get_runtime(id).next_event() {
    serialize_to_buffer(event, buffer, length);
  } else {
    zero(length)
  }
}

#[no_mangle]
pub extern "C" fn set_choice(id: usize, choice_id: usize) -> bool {
  if invalid_runtime(id) { return false; }

  unsafe {
    match RUNTIMES[id].set_choice(choice_id) {
      Ok(_) => return true,
      Err(_) => return false,
    }
  }
}

fn rust_string(string: Cstring) -> String {
  unsafe { CStr::from_ptr(string) }.to_str().unwrap_or_default().to_string()
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
  if buffer.is_null() { return; }
  if length.is_null() { return; }
  let serialized = serialize(element).unwrap_or_default();
  unsafe {
    *length = serialized.len();
    std::ptr::copy_nonoverlapping(serialized.as_ptr(), buffer, serialized.len());
  }
}

fn zero(length: *mut usize) {
  if length.is_null() { return; }
  unsafe { *length = 0; }
}