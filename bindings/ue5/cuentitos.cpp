#include <dlfcn.h>
#include<stdio.h>
#include <cstdint>
#include <cstdlib>

void (*foo)(void);

size_t (*load_database)(uint8_t*, size_t);
void (*debug_db)(size_t);

int main() {
  void* handle = dlopen("../../target/debug/libcuentitos_runtime_c.so", RTLD_LAZY);
  
  load_database = (size_t (*)(uint8_t*, size_t))dlsym(handle, "load_database");
  debug_db = (void (*)(size_t))dlsym(handle, "debug_db");


  FILE * fp;
  fp = fopen ("../../../mr-nuggets-events/cuentitos.db", "r");
  
  fseek(fp, 0L, SEEK_END);
  size_t size = 0;
  size = ftell(fp);
  fseek(fp, 0L, SEEK_SET);

  uint8_t * buffer = NULL;
  buffer = (uint8_t*)malloc(size * sizeof(uint8_t));

  fread(buffer,sizeof(uint8_t),size,fp);
  
  fclose(fp);

  size_t db = load_database(buffer, size);

  debug_db(db);
  // Runtime* runtime = new_runtime(db);

  // get_event(runtime);
  // ? results = set_choice(runtime, choice_id);


  return 0;
}
