#pragma once
#include "NODE.hpp"
#include "../Scope.hpp"
#include "../State.hpp"
#include <string>

namespace nucleoid {

class FOR : public NODE {
public:
    std::string initExpr;
    std::string condExpr;
    std::string updateExpr;
    NODE*       body = nullptr;

    explicit FOR(const std::string& key) : NODE(key) { type = "FOR"; }

    void before(Scope* scope) override {}

    NODE* run(Scope* scope) override {
        // Execute init; the body expansion is handled by Stack.
        if (!initExpr.empty())
            state::expression(scope, initExpr);
        return body;
    }

    bool checkCondition(Scope* scope) {
        if (condExpr.empty()) return true;
        nlohmann::json v = state::expression(scope, condExpr);
        return v.is_boolean() ? v.get<bool>() : !v.is_null();
    }

    void step(Scope* scope) {
        if (!updateExpr.empty())
            state::expression(scope, updateExpr);
    }
};

} // namespace nucleoid
