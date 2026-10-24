#pragma once
// Portable versions of the GCC/Clang builtins the engine uses, for MSVC.
#include <cstddef>
#include <cstdint>
#include <cstring>

#if defined(_MSC_VER) && !defined(__clang__)
#include <intrin.h>
#include <xmmintrin.h>

inline int me_ctz32(uint32_t x) {
    unsigned long i;
    _BitScanForward(&i, x);
    return static_cast<int>(i);
}
inline int me_ctz64(uint64_t x) {
    unsigned long i;
    _BitScanForward64(&i, x);
    return static_cast<int>(i);
}
#define __builtin_ctz(x) me_ctz32(static_cast<uint32_t>(x))
#define __builtin_ctzll(x) me_ctz64(static_cast<uint64_t>(x))
#define __builtin_prefetch(addr, ...) _mm_prefetch(reinterpret_cast<const char*>(addr), _MM_HINT_T0)
#define __builtin_expect(x, y) (x)
#endif

#if defined(_WIN32)
/// memmem is a GNU/BSD extension; Windows' CRT lacks it.
inline void* memmem(const void* hay, size_t hayLen, const void* needle, size_t needleLen) {
    if (needleLen == 0) return const_cast<void*>(hay);
    if (hayLen < needleLen) return nullptr;
    const auto* h = static_cast<const unsigned char*>(hay);
    const auto* n = static_cast<const unsigned char*>(needle);
    const unsigned char first = n[0];
    const size_t last = hayLen - needleLen;
    for (size_t i = 0; i <= last; i++) {
        if (h[i] == first && std::memcmp(h + i, n, needleLen) == 0) return const_cast<unsigned char*>(h + i);
    }
    return nullptr;
}
#endif
