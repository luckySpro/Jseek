#include "Compat.h"

#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <cerrno>
#include <fcntl.h>
#include <io.h>
#include <string>

namespace {
std::wstring widen(const char* utf8) {
    int n = MultiByteToWideChar(CP_UTF8, 0, utf8, -1, nullptr, 0);
    std::wstring w(n > 0 ? n - 1 : 0, L'\0');
    if (n > 1) MultiByteToWideChar(CP_UTF8, 0, utf8, -1, w.data(), n);
    return w;
}
} // namespace

int truncate(const char* path, long long len) {
    HANDLE h = CreateFileW(widen(path).c_str(), GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE, nullptr,
                           OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (h == INVALID_HANDLE_VALUE) return -1;
    LARGE_INTEGER pos;
    pos.QuadPart = len;
    bool ok = SetFilePointerEx(h, pos, nullptr, FILE_BEGIN) && SetEndOfFile(h);
    CloseHandle(h);
    return ok ? 0 : -1;
}

FILE* me::fopenShared(const char* path, const char* mode) {
    std::string m(mode);
    const bool plus = m.find('+') != std::string::npos;
    const bool binary = m.find('b') != std::string::npos;
    DWORD access = 0, disposition = 0;
    int flags = binary ? _O_BINARY : _O_TEXT;
    switch (m[0]) {
    case 'r':
        access = GENERIC_READ | (plus ? GENERIC_WRITE : 0);
        disposition = OPEN_EXISTING;
        flags |= plus ? _O_RDWR : _O_RDONLY;
        break;
    case 'w':
        access = GENERIC_WRITE | (plus ? GENERIC_READ : 0);
        disposition = CREATE_ALWAYS;
        flags |= plus ? _O_RDWR : _O_WRONLY;
        break;
    case 'a':
        access = GENERIC_WRITE | (plus ? GENERIC_READ : 0);
        disposition = OPEN_ALWAYS;
        flags |= (plus ? _O_RDWR : _O_WRONLY) | _O_APPEND;
        break;
    default:
        return nullptr;
    }
    HANDLE h = CreateFileW(widen(path).c_str(), access, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                           nullptr, disposition, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (h == INVALID_HANDLE_VALUE) {
        errno = (GetLastError() == ERROR_FILE_NOT_FOUND || GetLastError() == ERROR_PATH_NOT_FOUND) ? ENOENT : EACCES;
        return nullptr;
    }
    int fd = _open_osfhandle(reinterpret_cast<intptr_t>(h), flags);
    if (fd < 0) {
        CloseHandle(h);
        return nullptr;
    }
    FILE* f = _fdopen(fd, mode);
    if (!f) _close(fd);
    if (f && m[0] == 'a') fseek(f, 0, SEEK_END);
    return f;
}

int me::replaceFile(const char* from, const char* to) {
    return MoveFileExW(widen(from).c_str(), widen(to).c_str(),
                       MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) ? 0 : -1;
}
#else
int me::replaceFile(const char* from, const char* to) {
    return std::rename(from, to);
}

FILE* me::fopenShared(const char* path, const char* mode) {
    return std::fopen(path, mode);
}
#endif
