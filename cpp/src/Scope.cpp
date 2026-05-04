#include "nucleoid/Scope.hpp"

namespace nucleoid {

Scope::Scope(Scope* p) : prior(p) {
    root = p ? p->root : this;
}

Scope* Scope::findOwner(const std::string& name) const {
    const Scope* idx = this;
    while (idx) {
        if (idx->graphFlags.count(name)) return const_cast<Scope*>(idx);
        idx = idx->prior;
    }
    return nullptr;
}

void Scope::assign(const std::string& name, const nlohmann::json& value) {
    graphFlags[name] = true;
    local[name]      = value;
}

nlohmann::json Scope::retrieve(const std::string& name) const {
    const Scope* idx = this;
    while (idx) {
        if (idx->graphFlags.count(name)) {
            auto it = idx->local.find(name);
            if (it != idx->local.end()) return it->second;
        }
        idx = idx->prior;
    }
    return nullptr;
}

NODE* Scope::getInstance() const {
    const Scope* idx = this;
    while (idx) {
        if (idx->instance) return idx->instance;
        idx = idx->prior;
    }
    return nullptr;
}

std::string Scope::retrieveObject() const {
    const Scope* idx = this;
    while (idx) {
        if (!idx->objectName.empty()) return idx->objectName;
        idx = idx->prior;
    }
    return {};
}

nlohmann::json Scope::retrieveInstance(const std::string& name) const {
    const Scope* idx = this;
    while (idx) {
        auto it = idx->instances.find(name);
        if (it != idx->instances.end()) return it->second;
        idx = idx->prior;
    }
    return nullptr;
}

} // namespace nucleoid
