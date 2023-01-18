#include <dlfcn.h>
#include<stdio.h>
#include <cstdint>
#include <cstdlib>

#include "cuentitos.h"
#include "mpack/mpack.h"

LoadDatabaseFn load_database;
DebugDbFn debug_db;
NewRuntimeFn new_runtime;
DebugRuntimeFn debug_runtime;
GetEventFn get_event;


int main() {
  void* handle = dlopen("../../target/debug/libcuentitos_runtime_c.so", RTLD_LAZY);
  
  load_database = (LoadDatabaseFn) dlsym(handle, "load_database");
  debug_db = (DebugDbFn) dlsym(handle, "debug_db");
  new_runtime = (NewRuntimeFn) dlsym(handle, "new_runtime");
  debug_runtime = (DebugRuntimeFn) dlsym(handle, "debug_runtime");
  get_event = (GetEventFn) dlsym(handle, "get_event");

  FILE * fp;
  fp = fopen ("../../../mr-nuggets-events/cuentitos.db", "r");
  
  fseek(fp, 0L, SEEK_END);
  size_t size = 0;
  size = ftell(fp);
  fseek(fp, 0L, SEEK_SET);

  uint8_t * buffer = (uint8_t*)malloc(size * sizeof(uint8_t));

  fread(buffer,sizeof(uint8_t),size,fp);
  
  fclose(fp);

  DatabaseId db_id = load_database(buffer, size);
  RuntimeId runtime_id = new_runtime(db_id);
  // debug_runtime(runtime_id);

  // Read next event
  uint8_t *event_buffer = (uint8_t*)malloc(10000 * sizeof(uint8_t));
  size_t* length = (size_t*)malloc(1 * sizeof(size_t));
  get_event(runtime_id, event_buffer, length);
}
