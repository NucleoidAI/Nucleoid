#pragma once
#include <memory>
#include <string>
#include <unordered_map>

namespace nucleoid {

// Forward-declared so NODE.hpp can include Graph.hpp without circularity.
class NODE;

// The knowledge graph: a flat map of key -> NODE*.
// Mirrors the `$` object in graph.ts.
class Graph {
public:
    static Graph& instance();

    void  store(const std::string& key, NODE* node);
    NODE* retrieve(const std::string& key) const;
    bool  has(const std::string& key) const;
    void  remove(const std::string& key);
    void  clear();

    const std::unordered_map<std::string, NODE*>& all() const { return nodes_; }

private:
    Graph() = default;
    std::unordered_map<std::string, NODE*> nodes_;
};

} // namespace nucleoid
