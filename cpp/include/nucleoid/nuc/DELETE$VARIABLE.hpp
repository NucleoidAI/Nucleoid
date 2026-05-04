#pragma once
#include "DELETE.hpp"
#include "../Graph.hpp"

namespace nucleoid {

class DELETE$VARIABLE : public DELETE {
public:
    explicit DELETE$VARIABLE(const std::string& key)
        : DELETE(key) { type = "DELETE_VAR"; }

    void graph(Scope* /*scope*/) override {
        NODE* node = Graph::instance().retrieve(target);
        if (!node) return;
        // Sever all outgoing edges from this node's predecessors.
        for (auto& [k, _] : node->previous) {
            NODE* pred = Graph::instance().retrieve(k);
            if (pred) pred->next.erase(node->key);
        }
        Graph::instance().remove(node->key);
    }
};

} // namespace nucleoid
