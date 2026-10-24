#include "jseek_c.h"
#include "DirectoryScanner.h"
#include "SearchEngine.h"
#include <string>
#include <vector>

struct jseek_engine {
    SearchEngine engine;
};

extern "C" {

jseek_engine* jseek_engine_new(void) {
    try { return new jseek_engine(); } catch (...) { return nullptr; }
}

void jseek_engine_free(jseek_engine* e) { delete e; }

uint64_t jseek_engine_scan(jseek_engine* e, const char* root) {
    if (!e || !root) return 0;
    try {
        DirectoryScanner scanner;
        scanner.scan(root);
        auto records = scanner.takeResults();
        e->engine.loadRecords(std::move(records));
        return e->engine.liveRecordCount();
    } catch (...) { return 0; }
}

int jseek_engine_load(jseek_engine* e, const char* path) {
    if (!e || !path) return 0;
    try { return e->engine.loadFromFile(path) ? 1 : 0; } catch (...) { return 0; }
}

int jseek_engine_save(const jseek_engine* e, const char* path) {
    if (!e || !path) return 0;
    try { return e->engine.saveToFile(path) ? 1 : 0; } catch (...) { return 0; }
}

uint64_t jseek_engine_count(const jseek_engine* e) {
    return e ? e->engine.liveRecordCount() : 0;
}

uint64_t jseek_engine_search(const jseek_engine* e, const char* query, uint32_t limit,
                             jseek_hit_cb cb, void* user) {
    if (!e || !query) return 0;
    try {
        QueryTimingInfo timing;
        std::vector<uint32_t> idx = e->engine.query(query, limit, true, timing);
        uint64_t n = 0;
        e->engine.forEachRecordWithPath(idx, [&](uint32_t, const FileRecord& r, const std::string& dir) {
            if (cb) {
                jseek_hit h{r.name.c_str(), dir.c_str(), r.type, r.size, static_cast<int64_t>(r.modTime)};
                cb(user, &h);
            }
            n++;
        });
        return n;
    } catch (...) { return 0; }
}

const char* jseek_engine_backend(void) {
#if defined(__APPLE__)
    return "getattrlistbulk";
#elif defined(_WIN32)
    return "std::filesystem";
#else
    return "posix";
#endif
}

} // extern "C"
