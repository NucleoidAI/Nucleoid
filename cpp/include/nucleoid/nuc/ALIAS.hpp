#pragma once
#include "NODE.hpp"
#include <string>

namespace nucleoid {

class ALIAS : public NODE {
public:
    std::string from;
    std::string to;

    explicit ALIAS(const std::string& key) : NODE(key) { type = "ALIAS"; }
};

} // namespace nucleoid
