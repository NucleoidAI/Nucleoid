#pragma once
#include "NODE.hpp"
#include "../JSEngine.hpp"
#include "../State.hpp"
#include <string>

namespace nucleoid {

class PROPERTY : public NODE {
public:
    std::string object;
    std::string property;
    std::string valueExpr;

    explicit PROPERTY(const std::string& key) : NODE(key) { type = "PROPERTY"; }

    NODE* run(Scope* scope) override {
        std::string code =
            "state." + object + "." + property + " = " + valueExpr + ";";
        JSEngine::instance().eval(code);
        return nullptr;
    }
};

} // namespace nucleoid
