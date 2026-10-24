#pragma once
#include <string>
#include <cstdint>

namespace me {
/// Case-insensitive lowercasing: ASCII fast-path (SIMD) + portable Unicode mapping.
std::string toLower(const std::string& s);
/// Lowercase a single code point (simple 1:1 mapping; unchanged if no mapping).
uint32_t lowerCodePoint(uint32_t c);
}
