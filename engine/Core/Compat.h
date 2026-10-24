#pragma once
// Small POSIX compatibility layer so the engine builds with MSVC on Windows.
// Include this instead of <unistd.h>.

#include <cstdio>
#include <ctime>

#if defined(_WIN32)
#include <io.h>
#include <fcntl.h>
#include <sys/utime.h>
#include <stdio.h>
#define popen _popen
#define pclose _pclose
#define utime _utime
#define utimbuf _utimbuf
#ifndef F_OK
#define F_OK 0
#endif
#ifndef R_OK
#define R_OK 4
#endif
#ifndef W_OK
#define W_OK 2
#endif
inline int fsync(int fd) { return _commit(fd); }
inline int access(const char* path, int mode) { return _access(path, mode); }
inline struct tm* localtime_r(const time_t* t, struct tm* out) { return localtime_s(out, t) == 0 ? out : nullptr; }
inline struct tm* gmtime_r(const time_t* t, struct tm* out) { return gmtime_s(out, t) == 0 ? out : nullptr; }
inline time_t timegm(struct tm* t) { return _mkgmtime(t); }
inline int ftruncate(int fd, long long len) { return _chsize_s(fd, len) == 0 ? 0 : -1; }
using off_t_compat = long long;
int truncate(const char* path, long long len); // implemented in Compat.cpp
#else
#include <unistd.h>
#endif

namespace me {
/// Atomically replace `to` with `from` (POSIX rename semantics). On Windows plain
/// rename() fails when `to` exists, so this uses MoveFileExW(MOVEFILE_REPLACE_EXISTING).
/// Returns 0 on success, like rename().
int replaceFile(const char* from, const char* to);
} // namespace me
