#pragma once
#include "NODE.hpp"
#include "../Graph.hpp"
#include "../Scope.hpp"
#include <string>
#include <unordered_map>

namespace nucleoid {

class OBJECT$CLASS : public NODE {
public:
    struct ClassRef {
        std::unordered_map<std::string, NODE*> instances;
        std::unordered_map<std::string, NODE*> declarations;
    };

    std::string name;
    ClassRef*   classRef = nullptr;

    explicit OBJECT$CLASS(const std::string& key) : NODE(key) { type = "CLASS"; }

    NODE* run(Scope* scope) override;

    void graph(Scope* /*scope*/) override {
        if (classRef) classRef->declarations[key] = this;
    }
};

} // namespace nucleoid
