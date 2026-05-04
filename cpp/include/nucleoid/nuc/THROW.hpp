#pragma once
#include "NODE.hpp"
#include "../Scope.hpp"
#include "../State.hpp"

namespace nucleoid {

class THROW : public NODE {
public:
    std::string expression; // JS expression string to throw

    explicit THROW(const std::string& key) : NODE(key) { type = "THROW"; }

    NODE* run(Scope* scope) override {
        state::throwException(scope, expression);
    }
};

} // namespace nucleoid
