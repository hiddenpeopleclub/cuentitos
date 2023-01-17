use cuentitos_common::Database;
use std::slice;


static mut DATABASES: Vec<Database> = vec![]; 
// static RUNTIMES: Vec<Runtime> = vec![];

#[no_mangle]
pub extern "C" fn load_database(buffer: *const u8, length: usize) -> usize {
  if buffer.is_null() { return 0; }

  let rust_buffer: &[u8] = unsafe { slice::from_raw_parts(buffer, length as usize) };
  let db = Database::from_u8(rust_buffer);

  match db {
    Ok(db) => {
      unsafe {
        DATABASES.push(db);
        return DATABASES.len() - 1;
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
