#pragma once
#include "Types.hpp"
#include <vector>

namespace nucleoid {

void              cacheInit(const Config& cfg);
void              cacheWrite(const Data& data);
std::vector<Data> cacheRead();
std::vector<Data> cacheTail(int n = 10);
void              cacheClear();

} // namespace nucleoid
