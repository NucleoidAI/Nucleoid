#pragma once
#include "NODE.hpp"
#include "../Scope.hpp"
#include <unordered_map>
#include <string>

namespace nucleoid {

class LET$CLASS : public NODE {
public:
    struct ClassRef {
        std::string                            name;
        std::unordered_map<std::string, NODE*> instances;
        std::unordered_map<std::string, NODE*> declarations;
    };

    std::string name;
    std::string valueExpr;
    ClassRef*   classRef = nullptr;

    explicit LET$CLASS(const std::string& key) : NODE(key) { type = "CLASS"; }

    void  before(Scope* /*scope*/) override {}
    NODE* run(Scope* scope) override;
};

} // namespace nucleoid
