#include "StringUtils.h"
#include "SIMDSearch.h"
#include <cctype>
#include <cstring>

namespace me {

namespace {

/// Decode one UTF-8 sequence at s[i]; len = bytes consumed, 0 if invalid.
uint32_t decodeUtf8(const std::string& s, size_t i, size_t& len) {
    auto b = [&](size_t k) { return static_cast<unsigned char>(s[k]); };
    unsigned char c = b(i);
    len = 0;
    if (c < 0x80) { len = 1; return c; }
    int n = (c >> 5) == 0x6 ? 2 : (c >> 4) == 0xE ? 3 : (c >> 3) == 0x1E ? 4 : 0;
    if (n == 0 || i + n > s.size()) return 0;
    uint32_t cp = c & (0xFF >> (n + 1));
    for (int k = 1; k < n; k++) {
        unsigned char cc = b(i + k);
        if ((cc & 0xC0) != 0x80) return 0;
        cp = (cp << 6) | (cc & 0x3F);
    }
    len = static_cast<size_t>(n);
    return cp;
}

void appendUtf8(std::string& out, uint32_t cp) {
    if (cp < 0x80) {
        out.push_back(static_cast<char>(cp));
    } else if (cp < 0x800) {
        out.push_back(static_cast<char>(0xC0 | (cp >> 6)));
        out.push_back(static_cast<char>(0x80 | (cp & 0x3F)));
    } else if (cp < 0x10000) {
        out.push_back(static_cast<char>(0xE0 | (cp >> 12)));
        out.push_back(static_cast<char>(0x80 | ((cp >> 6) & 0x3F)));
        out.push_back(static_cast<char>(0x80 | (cp & 0x3F)));
    } else {
        out.push_back(static_cast<char>(0xF0 | (cp >> 18)));
        out.push_back(static_cast<char>(0x80 | ((cp >> 12) & 0x3F)));
        out.push_back(static_cast<char>(0x80 | ((cp >> 6) & 0x3F)));
        out.push_back(static_cast<char>(0x80 | (cp & 0x3F)));
    }
}

} // namespace

/// Simple (1:1) Unicode lowercase mapping for the scripts that occur in file names:
/// Latin (incl. Extended-A/B and Additional), Greek, Cyrillic, Armenian, Georgian,
/// Cherokee-free ranges, and fullwidth Latin. CJK, kana, digits and symbols are unchanged.
uint32_t lowerCodePoint(uint32_t c) {
    if (c < 0x80) return (c >= 'A' && c <= 'Z') ? c + 32 : c;
    // Latin-1 Supplement
    if ((c >= 0xC0 && c <= 0xDE) && c != 0xD7) return c + 32;
    // Latin Extended-A: pairs (even upper, odd lower) with exceptions
    if (c >= 0x0100 && c <= 0x017F) {
        if (c == 0x0130) return 0x0069;            // İ -> i
        if (c == 0x0178) return 0x00FF;            // Ÿ -> ÿ
        if ((c >= 0x0139 && c <= 0x0148) || (c >= 0x0179 && c <= 0x017E))
            return (c % 2 == 1) ? c + 1 : c;       // odd upper in these runs
        if (c == 0x0138 || c == 0x0149 || c == 0x017F) return c;
        return (c % 2 == 0) ? c + 1 : c;
    }
    // Latin Extended-B (common pairs)
    if (c >= 0x01CD && c <= 0x01DC) return (c % 2 == 1) ? c + 1 : c;
    if (c >= 0x01DE && c <= 0x01EF) return (c % 2 == 0) ? c + 1 : c;
    if ((c >= 0x01F8 && c <= 0x021F) || (c >= 0x0222 && c <= 0x0233) || (c >= 0x0246 && c <= 0x024F))
        return (c % 2 == 0) ? c + 1 : c;
    // Greek
    if (c == 0x0386) return 0x03AC;
    if (c >= 0x0388 && c <= 0x038A) return c + 37;
    if (c == 0x038C) return 0x03CC;
    if (c == 0x038E || c == 0x038F) return c + 63;
    if (c >= 0x0391 && c <= 0x03AB && c != 0x03A2) return c + 32;
    // Cyrillic
    if (c >= 0x0400 && c <= 0x040F) return c + 80;
    if (c >= 0x0410 && c <= 0x042F) return c + 32;
    if ((c >= 0x0460 && c <= 0x0481) || (c >= 0x048A && c <= 0x04BF) ||
        (c >= 0x04D0 && c <= 0x052F))
        return (c % 2 == 0) ? c + 1 : c;
    if (c == 0x04C0) return 0x04CF;
    if (c >= 0x04C1 && c <= 0x04CE) return (c % 2 == 1) ? c + 1 : c;
    // Armenian
    if (c >= 0x0531 && c <= 0x0556) return c + 48;
    // Georgian (Asomtavruli -> Nuskhuri)
    if (c >= 0x10A0 && c <= 0x10C5) return c + 7264;
    // Latin Extended Additional, Greek Extended (pairs)
    if (c >= 0x1E00 && c <= 0x1E95) return (c % 2 == 0) ? c + 1 : c;
    if (c == 0x1E9E) return 0x00DF;                // ẞ -> ß
    if (c >= 0x1EA0 && c <= 0x1EFF) return (c % 2 == 0) ? c + 1 : c;
    // Roman numerals, circled letters
    if (c >= 0x2160 && c <= 0x216F) return c + 16;
    if (c >= 0x24B6 && c <= 0x24CF) return c + 26;
    // Fullwidth Latin
    if (c >= 0xFF21 && c <= 0xFF3A) return c + 32;
    return c;
}

std::string toLower(const std::string& s) {
    // ASCII fast-path for pure-ASCII strings
    // (>95% of filenames on typical systems). Uses NEON SIMD on ARM64.
    bool allAscii = true;
    for (unsigned char c : s) {
        if (c >= 128) { allAscii = false; break; }
    }
    if (allAscii) {
        std::string result = s;
        simdToLowerAscii(result.data(), result.size());
        return result;
    }

    // Unicode-aware lowercasing (portable; replaces CoreFoundation).
    std::string result;
    result.reserve(s.size());
    size_t i = 0;
    while (i < s.size()) {
        size_t len = 0;
        uint32_t cp = decodeUtf8(s, i, len);
        if (len == 0) {               // invalid byte: copy through unchanged
            result.push_back(s[i]);
            i += 1;
            continue;
        }
        appendUtf8(result, lowerCodePoint(cp));
        i += len;
    }
    return result;
}

} // namespace me
