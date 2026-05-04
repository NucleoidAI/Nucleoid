#pragma once
#include "VARIABLE.hpp"

namespace nucleoid {
// LET is a block-scoped variable — structurally identical to VARIABLE
// in this runtime; scope management is handled by Scope itself.
class LET : public VARIABLE {
public:
    explicit LET(const std::string& key) : VARIABLE(key) { type = "LET"; }
};
} // namespace nucleoid
