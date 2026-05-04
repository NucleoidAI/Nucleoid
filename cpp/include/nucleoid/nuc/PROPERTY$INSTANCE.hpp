#pragma once
#include "PROPERTY.hpp"
#include "../Scope.hpp"

namespace nucleoid {

class PROPERTY$INSTANCE : public PROPERTY {
public:
    NODE*       instanceNode = nullptr;
    std::string className;

    explicit PROPERTY$INSTANCE(const std::string& key)
        : PROPERTY(key) { type = "INSTANCE"; }

    void before(Scope* scope) override {
        // Identifier substitution handled at JS-eval time.
        PROPERTY::before(scope);
    }
};

} // namespace nucleoid
