#pragma once
#include "NODE.hpp"

namespace nucleoid {
class BREAK : public NODE {
public:
    explicit BREAK(const std::string& key) : NODE(key) { type = "BREAK"; }
};
} // namespace nucleoid
