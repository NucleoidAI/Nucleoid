#pragma once
#include "NODE.hpp"
#include <string>
#include <vector>

namespace nucleoid {

class FUNCTION : public NODE {
public:
    std::string              name;
    std::vector<std::string> params;
    NODE*                    body = nullptr;

    explicit FUNCTION(const std::string& key) : NODE(key) { type = "FUNCTION"; }
};

} // namespace nucleoid
