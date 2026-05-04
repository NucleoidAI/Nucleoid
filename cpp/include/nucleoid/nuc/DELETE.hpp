#pragma once
#include "NODE.hpp"
#include "../State.hpp"
#include <string>

namespace nucleoid {

class DELETE : public NODE {
public:
    std::string target; // JS path to delete (e.g. "myVar" or "obj.prop")

    explicit DELETE(const std::string& key) : NODE(key) { type = "DELETE"; }

    NODE* run(Scope* scope) override {
        state::del(scope, target);
        return nullptr;
    }
};

} // namespace nucleoid
