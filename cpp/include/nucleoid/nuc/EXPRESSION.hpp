#pragma once
#include "NODE.hpp"
#include "../JSEngine.hpp"
#include "../Scope.hpp"
#include "../State.hpp"
#include <string>
#include <vector>

namespace nucleoid {

class EXPRESSION : public NODE {
public:
    std::string  expr;    // raw JS expression string
    bool         wrt = false; // write-flag (used for graph result collection)
    std::string  iof  = "EXPRESSION";

    explicit EXPRESSION(const std::string& key, const std::string& expression)
        : NODE(key), expr(expression) { type = "EXPRESSION"; }

    void before(Scope* scope) override {}

    NODE* run(Scope* scope) override {
        try {
            lastValue = state::expression(scope, expr);
        } catch (...) {
            lastValue = nullptr;
        }
        return nullptr;
    }

    // Overload used directly by Stack with extra hints (non-virtual).
    NODE* run(Scope* scope, bool /*force*/, bool /*dep*/) {
        return run(scope);
    }

    void graph(Scope* scope) override {}

    nlohmann::json lastValue = nullptr;

    // next() equivalent: returns dependent nodes from graph.
    std::vector<NODE*> nextNodes() const { return {}; }
};

} // namespace nucleoid
