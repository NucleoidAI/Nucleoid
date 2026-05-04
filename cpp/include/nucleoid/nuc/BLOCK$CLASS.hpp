#pragma once
#include "NODE.hpp"
#include "../Instruction.hpp"
#include "../Scope.hpp"
#include <string>
#include <unordered_map>
#include <vector>

namespace nucleoid {

class BLOCK;

class BLOCK$CLASS : public NODE {
public:
    struct ClassRef {
        std::unordered_map<std::string, NODE*> instances;
        std::unordered_map<std::string, NODE*> declarations;
    };

    std::vector<NODE*> statements;
    ClassRef*          classRef = nullptr;
    std::string        id;

    explicit BLOCK$CLASS(const std::string& key) : NODE(key) { type = "CLASS"; }

    NODE* run(Scope* scope) override;

    void graph(Scope* /*scope*/) override {
        if (classRef) classRef->declarations[id] = this;
    }
};

} // namespace nucleoid
