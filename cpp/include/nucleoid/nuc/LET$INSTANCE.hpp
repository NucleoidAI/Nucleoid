#pragma once
#include "LET.hpp"
#include "../Scope.hpp"

namespace nucleoid {

class LET$INSTANCE : public LET {
public:
    NODE*       instanceNode = nullptr;
    std::string className;

    explicit LET$INSTANCE(const std::string& key) : LET(key) { type = "INSTANCE"; }

    void before(Scope* scope) override {
        // Instance identifier substitution is handled at JS eval level.
        LET::before(scope);
    }
};

} // namespace nucleoid
