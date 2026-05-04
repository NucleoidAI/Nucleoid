#pragma once
#include "NODE.hpp"
#include "../Scope.hpp"
#include <string>
#include <unordered_map>

namespace nucleoid {

class PROPERTY$CLASS : public NODE {
public:
    struct ClassRef {
        std::unordered_map<std::string, NODE*> instances;
        std::unordered_map<std::string, NODE*> declarations;
    };

    std::string name;
    std::string valueExpr;
    ClassRef*   classRef = nullptr;

    explicit PROPERTY$CLASS(const std::string& key) : NODE(key) { type = "CLASS"; }

    NODE* run(Scope* scope) override;

    void graph(Scope* /*scope*/) override {
        if (classRef) classRef->declarations[key] = this;
    }
};

} // namespace nucleoid
