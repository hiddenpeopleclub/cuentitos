#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum class TimeOfDay {
  Morning,
  Noon,
  Evening,
  Night,
};

using DatabaseId = uintptr_t;

using RuntimeId = uintptr_t;

using Cstring = const char*;

extern "C" {

DatabaseId cuentitos_load_db(const uint8_t *buffer, uintptr_t length);

RuntimeId cuentitos_new_runtime(DatabaseId id);

void cuentitos_set_seed(RuntimeId id, uint64_t seed);

bool cuentitos_set_int_resource(RuntimeId id, Cstring resource, int32_t value);

bool cuentitos_set_float_resource(RuntimeId id, Cstring resource, float value);

bool cuentitos_set_bool_resource(RuntimeId id, Cstring resource, bool value);

bool cuentitos_get_int_resource(RuntimeId id, Cstring resource, int32_t *value);

bool cuentitos_get_float_resource(RuntimeId id, Cstring resource, float *value);

bool cuentitos_get_bool_resource(RuntimeId id, Cstring resource, bool *value);

bool cuentitos_set_item(RuntimeId id, Cstring item, uint8_t value);

bool cuentitos_set_time_of_day(RuntimeId id, TimeOfDay time_of_day);

bool cuentitos_set_tile(RuntimeId id, Cstring tile);

bool cuentitos_get_reputation(RuntimeId id, Cstring reputation, int32_t *value);

bool cuentitos_get_decision(RuntimeId id, Cstring decision, bool *value);

void cuentitos_next_event(uintptr_t id, uint8_t *buffer, uintptr_t *length);

bool cuentitos_set_choice(uintptr_t id, int32_t choice_id, uint8_t *buffer, uintptr_t *length);

bool cuentitos_current_modifiers(uintptr_t id, uint8_t *buffer, uintptr_t *length);

} // extern "C"
