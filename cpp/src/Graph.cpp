#include "nucleoid/Graph.hpp"

namespace nucleoid {

Graph& Graph::instance() {
    static Graph g;
    return g;
}

void Graph::store(const std::string& key, NODE* node) {
    nodes_[key] = node;
}

NODE* Graph::retrieve(const std::string& key) const {
    auto it = nodes_.find(key);
    return it != nodes_.end() ? it->second : nullptr;
}

bool Graph::has(const std::string& key) const {
    return nodes_.count(key) > 0;
}

void Graph::remove(const std::string& key) {
    nodes_.erase(key);
}

void Graph::clear() {
    nodes_.clear();
    // Preserve the "classes" sentinel like the TS version does.
    nodes_["classes"] = nullptr;
}

} // namespace nucleoid
