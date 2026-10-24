#if !defined(__APPLE__)
// Portable DirectoryScanner for Linux and Windows (macOS uses getattrlistbulk in
// DirectoryScanner.cpp). Same record semantics: one record per entry with its parent
// directory as `path`; directories are deduplicated, cross-mount directories are skipped,
// ".app" bundles are recorded as type 5 and not descended into, symlinks are not followed.

#include "DirectoryScanner.h"
#include "Logger.h"
#include <cctype>
#include <cstring>
#include <filesystem>
#include <system_error>

#if defined(_WIN32)
#include <chrono>
#else
#include <dirent.h>
#include <sys/stat.h>
#endif

namespace fs = std::filesystem;

namespace {

bool isAppBundleName(const std::string& name) {
    size_t n = name.size();
    return n > 4 && name[n - 4] == '.' && std::tolower(static_cast<unsigned char>(name[n - 3])) == 'a' &&
           std::tolower(static_cast<unsigned char>(name[n - 2])) == 'p' &&
           std::tolower(static_cast<unsigned char>(name[n - 1])) == 'p';
}

std::string joinPath(const std::string& dir, const std::string& name) {
    if (dir.empty()) return name;
    char last = dir.back();
    if (last == '/' || last == '\\') return dir + name;
    return dir + '/' + name;
}

} // namespace

void DirectoryScanner::scan(const std::string& rootPath) {
    done_.store(false, std::memory_order_relaxed);
    cancelled_.store(false, std::memory_order_relaxed);
    activeTasks_.store(0, std::memory_order_relaxed);
    {
        std::lock_guard<std::mutex> lock(dedupMutex_);
        visitedDirs_.clear();
    }
    {
        std::lock_guard<std::mutex> lock(queueMutex_);
        while (!workQueue_.empty()) workQueue_.pop();
    }
    threadResults_.clear();
    threadResults_.resize(1);
    stats_.fileCount.store(0);
    stats_.dirCount.store(0);
    stats_.symlinkCount.store(0);
    stats_.otherCount.store(0);
    stats_.errorCount.store(0);

#if !defined(_WIN32)
    struct stat rootStat;
    rootDevId_ = (stat(rootPath.c_str(), &rootStat) == 0) ? rootStat.st_dev : 0;
#else
    rootDevId_ = 0;
#endif
    LOG_INFO("Scanner", "Scanning from: " << rootPath << " (portable scanner)");

#if defined(_WIN32)
    // The engine stores paths with '/' separators on every platform.
    std::string root = rootPath;
    for (auto& ch : root) if (ch == '\\') ch = '/';
    workQueue_.push(root);
#else
    workQueue_.push(rootPath);
#endif
    std::vector<char> unused;
    while (!workQueue_.empty()) {
        if (cancelled_.load(std::memory_order_relaxed)) break;
        std::string dir = std::move(workQueue_.front());
        workQueue_.pop();
        scanDirectory(dir, unused.data(), 0);
    }
    done_.store(true);
}

std::vector<FileRecord> DirectoryScanner::takeResults() {
    std::vector<FileRecord> merged;
    for (auto& v : threadResults_) {
        merged.insert(merged.end(), std::make_move_iterator(v.begin()), std::make_move_iterator(v.end()));
    }
    threadResults_.clear();
    return merged;
}

void DirectoryScanner::workerThread(int) {}

#if !defined(_WIN32)
void DirectoryScanner::scanDirectory(const std::string& dirPath, char*, int threadIndex) {
    DIR* d = opendir(dirPath.c_str());
    if (!d) {
        stats_.errorCount.fetch_add(1, std::memory_order_relaxed);
        return;
    }
    auto& out = threadResults_[threadIndex];
    while (dirent* e = readdir(d)) {
        if (cancelled_.load(std::memory_order_relaxed)) break;
        const char* name = e->d_name;
        if (name[0] == '\0' || std::strcmp(name, ".") == 0 || std::strcmp(name, "..") == 0) continue;
        std::string child = joinPath(dirPath, name);
        struct stat st;
        if (lstat(child.c_str(), &st) != 0) {
            stats_.errorCount.fetch_add(1, std::memory_order_relaxed);
            continue;
        }
        const time_t mtime = st.st_mtime;
        const uint64_t ino = static_cast<uint64_t>(st.st_ino);
        const int32_t dev = static_cast<int32_t>(st.st_dev);
        if (S_ISDIR(st.st_mode)) {
            if (rootDevId_ != 0 && st.st_dev != rootDevId_) continue;
            if (!tryVisitDirectory(st.st_dev, ino)) continue;
            bool app = isAppBundleName(name);
            if (!app) workQueue_.push(child);
            stats_.dirCount.fetch_add(1, std::memory_order_relaxed);
            out.push_back({name, dirPath, static_cast<uint8_t>(app ? 5 : 2), 0, mtime, ino, dev});
        } else if (S_ISREG(st.st_mode)) {
            stats_.fileCount.fetch_add(1, std::memory_order_relaxed);
            out.push_back({name, dirPath, 1, static_cast<uint64_t>(st.st_size), mtime, ino, dev});
        } else if (S_ISLNK(st.st_mode)) {
            stats_.symlinkCount.fetch_add(1, std::memory_order_relaxed);
            out.push_back({name, dirPath, 3, 0, mtime, ino, dev});
        } else {
            stats_.otherCount.fetch_add(1, std::memory_order_relaxed);
            out.push_back({name, dirPath, 4, 0, mtime, ino, dev});
        }
    }
    closedir(d);
}
#else
void DirectoryScanner::scanDirectory(const std::string& dirPath, char*, int threadIndex) {
    std::error_code ec;
    fs::directory_iterator it(fs::path(std::u8string(dirPath.begin(), dirPath.end())), fs::directory_options::skip_permission_denied, ec);
    if (ec) {
        stats_.errorCount.fetch_add(1, std::memory_order_relaxed);
        return;
    }
    auto& out = threadResults_[threadIndex];
    for (; it != fs::directory_iterator(); it.increment(ec)) {
        if (ec || cancelled_.load(std::memory_order_relaxed)) break;
        const fs::directory_entry& e = *it;
        auto u8name = e.path().filename().u8string();
        std::string name(u8name.begin(), u8name.end());
        if (name.empty()) continue;
        std::string child = joinPath(dirPath, name);
        std::error_code ec2;
        auto st = e.symlink_status(ec2);
        if (ec2) {
            stats_.errorCount.fetch_add(1, std::memory_order_relaxed);
            continue;
        }
        time_t mtime = 0;
        auto ft = e.last_write_time(ec2);
        if (!ec2) {
            auto sys = std::chrono::time_point_cast<std::chrono::seconds>(
                ft - fs::file_time_type::clock::now() + std::chrono::system_clock::now());
            mtime = static_cast<time_t>(sys.time_since_epoch().count());
        }
        if (fs::is_directory(st)) {
            // No inode numbers here: dedupe by a hash of the full path.
            uint64_t key = std::hash<std::string>{}(child);
            if (!tryVisitDirectory(0, key)) continue;
            bool app = isAppBundleName(name);
            if (!app) workQueue_.push(child);
            stats_.dirCount.fetch_add(1, std::memory_order_relaxed);
            out.push_back({name, dirPath, static_cast<uint8_t>(app ? 5 : 2), 0, mtime, 0, 0});
        } else if (fs::is_regular_file(st)) {
            uint64_t size = e.file_size(ec2);
            stats_.fileCount.fetch_add(1, std::memory_order_relaxed);
            out.push_back({name, dirPath, 1, ec2 ? 0 : size, mtime, 0, 0});
        } else if (fs::is_symlink(st)) {
            stats_.symlinkCount.fetch_add(1, std::memory_order_relaxed);
            out.push_back({name, dirPath, 3, 0, mtime, 0, 0});
        } else {
            stats_.otherCount.fetch_add(1, std::memory_order_relaxed);
            out.push_back({name, dirPath, 4, 0, mtime, 0, 0});
        }
    }
}
#endif

bool DirectoryScanner::tryVisitDirectory(dev_t dev, uint64_t ino) {
    std::lock_guard<std::mutex> lock(dedupMutex_);
    return visitedDirs_.insert({dev, ino}).second;
}

#endif // !__APPLE__
