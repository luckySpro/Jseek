/* Plain C ABI over the C++ search engine, consumed by the Rust crate `jseek-engine`.
 * A C ABI (rather than cxx) keeps the bridge dependency-free and identical on all three
 * platforms (decision D-008). All strings are UTF-8. Paths use '/' separators. */
#pragma once
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

typedef struct jseek_engine jseek_engine;

typedef struct jseek_hit {
    const char* name;     /* valid only during the callback */
    const char* dir;      /* parent directory */
    uint8_t     kind;     /* 1 file, 2 dir, 3 symlink, 4 other, 5 app bundle */
    uint64_t    size;
    int64_t     modified; /* seconds since the Unix epoch */
} jseek_hit;

typedef void (*jseek_hit_cb)(void* user, const jseek_hit* hit);

jseek_engine* jseek_engine_new(void);
void          jseek_engine_free(jseek_engine* e);

/* Full scan of `root` into the engine (replaces existing records). Returns record count. */
uint64_t jseek_engine_scan(jseek_engine* e, const char* root);
/* Load / save an index file. Return 1 on success, 0 on failure. */
int      jseek_engine_load(jseek_engine* e, const char* path);
int      jseek_engine_save(const jseek_engine* e, const char* path);
uint64_t jseek_engine_count(const jseek_engine* e);

/* Run a query with the full 1.9 syntax (ext:, size:, parent:, AND/OR/NOT, ...).
 * Calls `cb` once per hit, in ranked order. Returns the number of hits. */
uint64_t jseek_engine_search(const jseek_engine* e, const char* query, uint32_t limit,
                             jseek_hit_cb cb, void* user);

/* Name of the scanning backend compiled in ("getattrlistbulk", "posix", "std::filesystem"). */
const char* jseek_engine_backend(void);

#ifdef __cplusplus
}
#endif
