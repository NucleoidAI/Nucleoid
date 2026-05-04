#include "nucleoid/nuc/PROPERTY$CLASS.hpp"
#include "nucleoid/nuc/PROPERTY$INSTANCE.hpp"
#include "nucleoid/Scope.hpp"
#include <cstdint>
#include <random>
#include <sstream>

namespace nucleoid {

static std::string propUUID() {
    static std::mt19937_64 rng(std::random_device{}());
    std::uniform_int_distribution<uint64_t> dist;
    std::ostringstream ss;
    ss << std::hex << dist(rng);
    return ss.str();
}

NODE* PROPERTY$CLASS::run(Scope* scope) {
    if (!classRef) return nullptr;

    NODE* inst = scope->instance;
    std::vector<NODE*> instances;
    if (inst) instances.push_back(inst);
    else for (auto& [k, v] : classRef->instances) if (v) instances.push_back(v);

    for (NODE* i : instances) {
        std::string instName = i->key;
        std::string id       = instName + "." + name;

        auto* pi = new PROPERTY$INSTANCE(id);
        pi->object       = instName;
        pi->property     = name;
        pi->valueExpr    = valueExpr;
        pi->instanceNode = i;

        Scope* s  = new Scope(scope);
        s->instance = i;

        return pi;
    }
    return nullptr;
}

} // namespace nucleoid
