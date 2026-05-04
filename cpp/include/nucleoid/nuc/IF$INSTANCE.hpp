#pragma once
#include "IF.hpp"
#include "../Scope.hpp"

namespace nucleoid {

class IF$INSTANCE : public IF {
public:
    NODE* instanceNode = nullptr;

    explicit IF$INSTANCE(const std::string& key) : IF(key) { type = "INSTANCE"; }

    void before(Scope* scope) override {
        // Substitute class-level references with the concrete instance path.
        // In C++ we handle this via the conditionExpr already being resolved
        // by IF$CLASS before handing off to IF$INSTANCE.
        IF::before(scope);
    }
};

} // namespace nucleoid
