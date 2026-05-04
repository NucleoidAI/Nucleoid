#pragma once
#include "Types.hpp"
#include <vector>

namespace nucleoid {

void              datastoreWrite(const Data& data);
std::vector<Data> datastoreAll();
void              datastoreClear();

} // namespace nucleoid
