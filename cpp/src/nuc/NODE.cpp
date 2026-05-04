#include "nucleoid/nuc/NODE.hpp"
#include "nucleoid/Graph.hpp"
#include "nucleoid/Transaction.hpp"
#include <stdexcept>

namespace nucleoid {

std::atomic<int> NODE::s_sequence{0};

NODE::NODE(const std::string& k)
    : key(k), sequence(s_sequence.fetch_add(1)) {}

void NODE::registerNode(const std::string& key, NODE* node) {
    Graph::instance().store(key, node);
}

void NODE::replaceNode(const std::string& srcKey, NODE* target) {
    NODE* src = Graph::instance().retrieve(srcKey);
    if (!src) return;

    // Transfer all outgoing edges from src to target.
    for (auto& [k, v] : src->next) {
        target->next[k] = v;
    }
    src->next.clear();

    // Remove incoming edges that pointed to src.
    for (auto& [k, v] : src->previous) {
        NODE* pred = Graph::instance().retrieve(k);
        if (pred) pred->next.erase(srcKey);
    }

    Graph::instance().store(srcKey, target);
}

void NODE::directNode(const std::string& srcKey,
                      const std::string& tgtKey, NODE* target) {
    NODE* src = Graph::instance().retrieve(srcKey);
    if (!src) return;
    src->next[tgtKey]      = target;
    target->previous[srcKey] = src;
}

} // namespace nucleoid
