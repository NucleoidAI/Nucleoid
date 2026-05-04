#pragma once
#include "NODE.hpp"
#include "../JSEngine.hpp"
#include "../Scope.hpp"
#include "../State.hpp"

namespace nucleoid {

class VARIABLE : public NODE {
public:
    std::string name;
    std::string valueExpr; // raw JS expression string for the initialiser

    explicit VARIABLE(const std::string& key) : NODE(key) { type = "VARIABLE"; }

    void before(Scope* scope) override {}

    NODE* run(Scope* scope) override {
        if (valueExpr.empty()) {
            state::assign(scope, name, nullptr);
            return nullptr;
        }
        nlohmann::json val = state::expression(scope, valueExpr);
        state::assign(scope, name, val);
        return nullptr;
    }

    void graph(Scope* scope) override {}
};

} // namespace nucleoid
