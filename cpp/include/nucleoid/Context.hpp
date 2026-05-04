#pragma once
#include <string>
#include <vector>

namespace nucleoid {

struct ContextItem {
    std::string definition;
    bool        declarative = false;
};

void contextLoad(const std::vector<ContextItem>& items);
void contextRun();

} // namespace nucleoid
