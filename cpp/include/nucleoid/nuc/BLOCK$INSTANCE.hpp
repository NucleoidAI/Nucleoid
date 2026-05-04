#pragma once
#include "BLOCK.hpp"
#include "../Scope.hpp"
#include <vector>

namespace nucleoid {

class BLOCK$INSTANCE : public BLOCK {
public:
    NODE*              instance    = nullptr;
    NODE*              declaration = nullptr;
    bool               brk        = false;

    explicit BLOCK$INSTANCE(const std::string& key) : BLOCK(key) { type = "INSTANCE"; }

    NODE* run(Scope* scope) override {
        scope->instance = instance;

        if (brk) {
            // Reload statements from parent declaration on break.
            if (declaration) {
                auto* decl = static_cast<BLOCK*>(declaration);
                body = decl->body;
            }
            brk = false;
        }
        return BLOCK::run(scope);
    }
};

} // namespace nucleoid
