#pragma once
#include "EXPRESSION.hpp"
#include "../Scope.hpp"

namespace nucleoid {

class EXPRESSION$INSTANCE : public EXPRESSION {
public:
    NODE*       instanceNode = nullptr;
    std::string className;

    explicit EXPRESSION$INSTANCE(const std::string& key, const std::string& expression)
        : EXPRESSION(key, expression) { iof = "EXPRESSION$INSTANCE"; }

    void before(Scope* scope) override {
        // Instance-level identifier substitution is handled at JS-eval time
        // since QuickJS already has the correct instance path in `state`.
        EXPRESSION::before(scope);
    }
};

} // namespace nucleoid
