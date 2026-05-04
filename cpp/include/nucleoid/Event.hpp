#pragma once
#include "Types.hpp"
#include <vector>

namespace nucleoid {

void               eventEmit(const std::string& topic, const std::string& data);
std::vector<Event> eventList();
void               eventClear();

} // namespace nucleoid
