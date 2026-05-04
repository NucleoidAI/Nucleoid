#pragma once
#include "NODE.hpp"

namespace nucleoid {
class RETURN : public NODE {
public:
    NODE* statement = nullptr; // the expression/value being returned

    explicit RETURN(const std::string& key) : NODE(key) { type = "RETURN"; }
};
} // namespace nucleoid
