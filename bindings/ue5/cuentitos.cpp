#include <dlfcn.h>
#include<stdio.h>

void (*foo)(void);

int main() {
  void* handle = dlopen("../../target/debug/libcuentitos_runtime_c.so", RTLD_LAZY);
  foo = (void (*)(void))dlsym(handle, "foo");

  // int size = size("file")
  // u8 buffer[size] = load_file("file");
  // Database* db = load_database(buffer);
  // Runtime* runtime = new_runtime(db);

  // get_event(runtime);
  // ? results = set_choice(runtime, choice_id);

  foo();
  
  return 0;
}
