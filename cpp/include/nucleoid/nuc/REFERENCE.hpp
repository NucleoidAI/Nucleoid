#pragma once
#include "NODE.hpp"
#include <string>

namespace nucleoid {

class REFERENCE : public NODE {
public:
    std::string target;   // the name this reference links to
    NODE*       link = nullptr;

    explicit REFERENCE(const std::string& key) : NODE(key) { type = "REFERENCE"; }
};

} // namespace nucleoid
