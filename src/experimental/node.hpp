#include "NODE.h"
#include <iostream>

int NODE::sequence = 0;
std::map<std::string, std::shared_ptr<NODE>> NODE::graph_nodes;

NODE::NODE(const std::string& key) : key(key) {
    my_sequence = sequence++;
}

void NODE::registerNode(const std::string& key, std::shared_ptr<NODE> node) {
    graph_nodes[key] = node;
}

void NODE::replace(const std::string& sourceKey, std::shared_ptr<NODE> targetNode) {
    auto& sourceNode = graph_nodes[sourceKey];
    targetNode->next = sourceNode->next;
    targetNode->previous = sourceNode->previous;

    for (auto& n : sourceNode->next) {
        graph_nodes[n.first]->previous[sourceKey] = targetNode;
    }

    for (auto& p : sourceNode->previous) {
        graph_nodes[p.first]->next[sourceKey] = targetNode;
    }

    graph_nodes[sourceKey] = targetNode;
}

void NODE::direct(const std::string& sourceKey, const std::string& targetKey, std::shared_ptr<NODE> targetNode) {
    graph_nodes[sourceKey]->next[targetKey] = targetNode;
    targetNode->previous[sourceKey] = graph_nodes[sourceKey];
}
