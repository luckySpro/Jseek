#include "Compat.h"

#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
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

int me::replaceFile(const char* from, const char* to) {
    return MoveFileExW(widen(from).c_str(), widen(to).c_str(),
                       MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) ? 0 : -1;
}
#else
int me::replaceFile(const char* from, const char* to) {
    return std::rename(from, to);
}
#endif
