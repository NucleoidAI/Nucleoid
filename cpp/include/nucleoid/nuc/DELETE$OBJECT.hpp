#pragma once
#include "DELETE.hpp"
#include "../Graph.hpp"
#include "../State.hpp"
#include <stdexcept>
#include <string>

namespace nucleoid {

class DELETE$OBJECT : public DELETE {
public:
    explicit DELETE$OBJECT(const std::string& key)
        : DELETE(key) { type = "DELETE_OBJ"; }

    NODE* run(Scope* scope) override {
        NODE* node = Graph::instance().retrieve(target);
        if (!node) return nullptr;

        // Verify the object has no child properties.
        bool hasProps = false;
        for (auto& [k, _] : Graph::instance().all()) {
            if (k.rfind(node->key + ".", 0) == 0) { hasProps = true; break; }
        }
        if (hasProps)
            throw std::runtime_error("Cannot delete object '" + target + "'");

        state::del(scope, node->key);
        return DELETE::run(scope);
    }

    void graph(Scope* /*scope*/) override {
        NODE* node = Graph::instance().retrieve(target);
        if (!node) return;
        for (auto& [k, _] : node->previous) {
            NODE* pred = Graph::instance().retrieve(k);
            if (pred) pred->next.erase(node->key);
        }
        Graph::instance().remove(node->key);
    }
};

} // namespace nucleoid
