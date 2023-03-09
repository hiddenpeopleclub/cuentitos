#include <dlfcn.h>
#include<stdio.h>
#include<stdint.h>

#include "cuentitos.hpp"

int main() {
  FILE * fp;
  fp = fopen ("../../../mr-nugget-events/cuentitos.db", "r");

  fseek(fp, 0L, SEEK_END);
  size_t size = 0;
  size = ftell(fp);
  fseek(fp, 0L, SEEK_SET);

  uint8_t * buffer = (uint8_t*)malloc(size * sizeof(uint8_t));

  fread(buffer,sizeof(uint8_t),size,fp);
  
  fclose(fp);

  DatabaseId db_id = cuentitos_load_db(buffer, size);
  RuntimeId runtime_id = cuentitos_new_runtime(db_id);

  cuentitos_set_seed(runtime_id, 42);
  
  cuentitos_set_int_variable(runtime_id, "health", 10);
  cuentitos_set_float_variable(runtime_id, "energized", 10.5);
  cuentitos_set_bool_variable(runtime_id, "donkey", true);

  int32_t health;
  float energized;
  bool donkey;

  cuentitos_get_int_variable(runtime_id, "health", &health);
  cuentitos_get_float_variable(runtime_id, "energized", &energized);
  cuentitos_get_bool_variable(runtime_id, "donkey", &donkey);

  // Read next event
  // uint8_t *event_buffer = (uint8_t*)malloc(10000 * sizeof(uint8_t));
  // size_t* length = (size_t*)malloc(1 * sizeof(size_t));
  // get_event(runtime_id, event_buffer, length);

  // // set choice
  // bool result = set_choice(runtime_id, 0);
  // if(result){
  //   printf("Choice 0 selected\n");
  // }else{
  //   printf("Error selecting choice\n");
  // }
}

