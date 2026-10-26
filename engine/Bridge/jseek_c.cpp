#include "jseek_c.h"
#include "DirectoryScanner.h"
#include "SearchEngine.h"
#include <string>
#include <vector>
#if defined(__APPLE__)
#include <mach/mach.h>
#include <malloc/malloc.h>
#elif defined(_WIN32)
#include <windows.h>
#include <psapi.h>
#include <malloc.h>
#else
#include <fstream>
#include <malloc.h>
#include <unistd.h>
#endif

struct jseek_engine {
    SearchEngine engine;
};

struct jseek_records {
    std::vector<FileRecord> records;
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

jseek_records* jseek_scan_records(const char* root, uint64_t* out_count) {
    if (out_count) *out_count = 0;
    if (!root) return nullptr;
    try {
        DirectoryScanner scanner;
        scanner.scan(root);
        auto* r = new jseek_records{scanner.takeResults()};
        if (out_count) *out_count = r->records.size();
        return r;
    } catch (...) { return nullptr; }
}

void jseek_engine_load_records(jseek_engine* e, jseek_records* r) {
    if (!r) return;
    if (e) {
        try { e->engine.loadRecords(std::move(r->records)); } catch (...) {}
    }
    delete r;
}

void jseek_records_free(jseek_records* r) { delete r; }

void jseek_release_memory(void) {
#if defined(__APPLE__)
    malloc_zone_pressure_relief(nullptr, 0);
#elif defined(_WIN32)
    _heapmin();
    HeapCompact(GetProcessHeap(), 0);
#elif defined(__GLIBC__)
    malloc_trim(0);
#endif
}

uint64_t jseek_memory_footprint(void) {
#if defined(__APPLE__)
    task_vm_info_data_t info;
    mach_msg_type_number_t count = TASK_VM_INFO_COUNT;
    if (task_info(mach_task_self(), TASK_VM_INFO, (task_info_t)&info, &count) == KERN_SUCCESS)
        return info.phys_footprint;
    return 0;
#elif defined(_WIN32)
    PROCESS_MEMORY_COUNTERS_EX pmc{};
    if (GetProcessMemoryInfo(GetCurrentProcess(), (PROCESS_MEMORY_COUNTERS*)&pmc, sizeof(pmc)))
        return pmc.PrivateUsage;
    return 0;
#else
    std::ifstream statm("/proc/self/statm");
    uint64_t pages = 0, resident = 0;
    if (statm >> pages >> resident) return resident * (uint64_t)sysconf(_SC_PAGESIZE);
    return 0;
#endif
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
