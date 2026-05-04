#pragma once
#include "NODE.hpp"
#include "../Scope.hpp"
#include "../State.hpp"
#include <string>

namespace nucleoid {

class IF : public NODE {
public:
    std::string conditionExpr;
    NODE*       trueBranch  = nullptr;
    NODE*       falseBranch = nullptr;

    explicit IF(const std::string& key) : NODE(key) { type = "IF"; }

    void before(Scope* scope) override {}

    NODE* run(Scope* scope) override {
        nlohmann::json cond = state::expression(scope, conditionExpr);
        if (cond.is_boolean() ? cond.get<bool>() : !cond.is_null()) {
            return trueBranch;
        }
        return falseBranch;
    }

    void graph(Scope* scope) override {}
};

} // namespace nucleoid
