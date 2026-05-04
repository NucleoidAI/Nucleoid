#pragma once
#include "NODE.hpp"
#include "../State.hpp"
#include <string>

namespace nucleoid {

class OBJECT : public NODE {
public:
    std::string className;
    std::string instanceName;

    explicit OBJECT(const std::string& key) : NODE(key) { type = "OBJECT"; }

    NODE* run(Scope* scope) override {
        std::string code =
            "state." + instanceName + " = new state." + className + "();";
        JSEngine::instance().eval(code);
        return nullptr;
    }
};

} // namespace nucleoid
