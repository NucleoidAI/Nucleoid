#pragma once
#include "LET.hpp"
#include "../Scope.hpp"
#include "../State.hpp"

namespace nucleoid {

class LET$OBJECT : public LET {
public:
    std::string objectKey; // key of the referenced object in the graph

    explicit LET$OBJECT(const std::string& key) : LET(key) { type = "OBJECT"; }

    void  before(Scope* /*scope*/) override {}

    NODE* run(Scope* scope) override {
        // Assign a reference to the object in state.
        state::assign(scope, name, "state." + objectKey);
        return nullptr;
    }

    void graph(Scope* /*scope*/) override {}
};

} // namespace nucleoid
