typedef size_t DatabaseId;
typedef size_t RuntimeId;

typedef DatabaseId(*LoadDatabaseFn)(uint8_t*, size_t);
typedef void(*DebugDbFn)(DatabaseId);
typedef RuntimeId(*NewRuntimeFn)(DatabaseId);
typedef void(*DebugRuntimeFn)(RuntimeId);
typedef void(*GetEventFn)(RuntimeId,uint8_t*, size_t*);
typedef void(*SetSeedFn)(RuntimeId,uint64_t);
typedef bool(*SetChoiceFn)(RuntimeId,size_t);
