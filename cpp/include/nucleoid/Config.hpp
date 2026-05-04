#pragma once
#include "Types.hpp"

namespace nucleoid {

void   configInit(const Config& cfg);
Config configGet();
void   configClear();

} // namespace nucleoid
