use cuentitos_runtime::Runtime;
use cuentitos_common::Database;
use std::slice;


static mut DATABASES: Vec<Database> = vec![]; 
static mut RUNTIMES: Vec<Runtime> = vec![];

#[no_mangle]
pub extern "C" fn load_database(buffer: *const u8, length: usize) -> usize {
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
pub extern "C" fn debug_db(id: usize) {
  unsafe {
    if DATABASES.len() > id {
      println!("{:?}", DATABASES[id]);
    }    
  }
}

#[no_mangle]
pub extern "C" fn new_runtime(id: usize) -> usize {
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

#[no_mangle]
pub extern "C" fn debug_runtime(id: usize) {
  unsafe {
    if RUNTIMES.len() > id {
      println!("{:?}", RUNTIMES[id]);
    }    
  }
}

#[no_mangle]
pub extern "C" fn get_event(runtime_id: usize, buffer: *mut u8, length: *mut usize) {
  unsafe {
    if RUNTIMES.len() > runtime_id {
      if let Some(event) = RUNTIMES[runtime_id].serialized_random_event() {
        *length = event.len();
        std::ptr::copy_nonoverlapping(event.as_ptr(), buffer, event.len());
      } else {
        *length = 0
      }
    }else{
      *length = 0;
    }
  }
}


