#pragma once
#include "NODE.hpp"
#include "../JSEngine.hpp"
#include "../State.hpp"
#include <string>

namespace nucleoid {

class OBJECT$INSTANCE : public NODE {
public:
    NODE*       classNode  = nullptr;
    NODE*       objectNode = nullptr;
    std::string name;

    explicit OBJECT$INSTANCE(const std::string& key)
        : NODE(key) { type = "INSTANCE"; }

    NODE* run(Scope* scope) override {
        if (objectNode && !name.empty()) {
            std::string code =
                "state." + objectNode->key + "." + name +
                " = new state." + name + "();";
            JSEngine::instance().eval(code);
        }
        return nullptr;
    }
};

} // namespace nucleoid
