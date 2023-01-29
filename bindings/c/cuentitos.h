typedef size_t DatabaseId;
typedef size_t RuntimeId;

// typedef void(*GetEventFn)(RuntimeId,uint8_t*, size_t*);
// typedef bool(*SetChoiceFn)(RuntimeId,size_t);

// Runtime Setup
DatabaseId cuentitos_load_db(uint8_t* buffer, size_t length);
RuntimeId  cuentitos_new_runtime(DatabaseId database_id);

// Runtime State
void cuentitos_set_seed(RuntimeId id, uint64_t seed);
bool cuentitos_set_int_resource(RuntimeId id, char* resource, int32_t value);
bool cuentitos_set_float_resource(RuntimeId id, char* resource, float value);
bool cuentitos_set_bool_resource(RuntimeId id, char* resource, bool value);
bool cuentitos_get_int_resource(RuntimeId id, char* resource, int32_t* value);
bool cuentitos_get_float_resource(RuntimeId id, char* resource, float* value);
bool cuentitos_get_bool_resource(RuntimeId id, char* resource, bool* value);

bool set_choice(RuntimeId id, size_t choice_id);

// pub fn set_resource<R, T>(&mut self, resource: R, value: T) -> Result<(), String>
// pub fn set_time_of_day(&mut self, time_of_day: TimeOfDay)
// pub fn get_reputation<T>(&self, reputation_id: T) -> Result<i32, String> 
// pub fn decision_taken<T>(&self, decision_id: T) -> bool
