#pragma once
#include "NODE.hpp"
#include "../Scope.hpp"
#include <unordered_map>
#include <vector>

namespace nucleoid {

class IF$CLASS : public NODE {
public:
    struct ClassRef {
        std::unordered_map<std::string, NODE*> instances;
        std::unordered_map<std::string, NODE*> declarations;
    };

    std::string conditionExpr;
    NODE*       trueBranch  = nullptr;
    NODE*       falseBranch = nullptr;
    ClassRef*   classRef    = nullptr;

    explicit IF$CLASS(const std::string& key) : NODE(key) { type = "CLASS"; }

    NODE* run(Scope* scope) override;

    void graph(Scope* /*scope*/) override {
        if (classRef) classRef->declarations[key] = this;
    }
};

} // namespace nucleoid
