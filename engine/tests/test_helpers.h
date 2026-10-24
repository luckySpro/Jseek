#pragma once
// ─────────── Helpers ───────────
#if defined(_WIN32)
#include <windows.h>
#include <psapi.h>
#else
#include <sys/resource.h>
#endif
#if defined(__APPLE__)
#include <mach/mach.h>
#endif
#include <fstream>

static size_t getMemoryUsageMB() {
#if defined(__APPLE__)
    mach_task_basic_info_data_t info;
    mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), MACH_TASK_BASIC_INFO, (task_info_t)&info, &count) == KERN_SUCCESS) {
        return info.resident_size / (1024 * 1024);
    }
    return 0;
#elif defined(_WIN32)
    PROCESS_MEMORY_COUNTERS pmc;
    if (GetProcessMemoryInfo(GetCurrentProcess(), &pmc, sizeof(pmc))) return pmc.WorkingSetSize / (1024 * 1024);
    return 0;
#else
    std::ifstream statm("/proc/self/statm");
    size_t pages = 0, resident = 0;
    if (statm >> pages >> resident) return resident * 4096 / (1024 * 1024);
    return 0;
#endif
}

struct CpuTime {
    double userMs;
    double sysMs;
    double totalMs() const { return userMs + sysMs; }
};

static CpuTime getCpuTime() {
#if defined(_WIN32)
    FILETIME create, exit, kernel, user;
    GetProcessTimes(GetCurrentProcess(), &create, &exit, &kernel, &user);
    auto ms = [](FILETIME f) { return ((static_cast<uint64_t>(f.dwHighDateTime) << 32) | f.dwLowDateTime) / 10000.0; };
    return {ms(user), ms(kernel)};
#else
    struct rusage ru;
    getrusage(RUSAGE_SELF, &ru);
    double userMs = ru.ru_utime.tv_sec * 1000.0 + ru.ru_utime.tv_usec / 1000.0;
    double sysMs  = ru.ru_stime.tv_sec * 1000.0 + ru.ru_stime.tv_usec / 1000.0;
    return {userMs, sysMs};
#endif
}

struct ResourceSnapshot {
    size_t memMB;
    CpuTime cpu;
    static ResourceSnapshot take() { return {getMemoryUsageMB(), getCpuTime()}; }
};

// Helper: derive paged persistence paths from a base path
static std::string pagesPathFor(const std::string& basePath) {
    auto dir = basePath.substr(0, basePath.rfind('/'));
    return dir + "/index.pages";
}
static std::string ptablePathFor(const std::string& basePath) {
    auto dir = basePath.substr(0, basePath.rfind('/'));
    return dir + "/index.ptable";
}

static int passed = 0, failed = 0;

#define CHECK(expr) check((expr), #expr)

static void check(bool cond, const char* msg) {
    if (cond) {
        std::cout << "    [PASS] " << msg << "\n";
        passed++;
    } else {
        std::cout << "    [FAIL] " << msg << "\n";
        failed++;
    }
}
