#include "nucleoid/nuc/OBJECT$CLASS.hpp"
#include "nucleoid/nuc/OBJECT$INSTANCE.hpp"
#include "nucleoid/Graph.hpp"
#include "nucleoid/Scope.hpp"
#include <cstdint>
#include <random>
#include <sstream>

namespace nucleoid {

static std::string objUUID() {
    static std::mt19937_64 rng(std::random_device{}());
    std::uniform_int_distribution<uint64_t> dist;
    std::ostringstream ss;
    ss << std::hex << dist(rng);
    return ss.str();
}

NODE* OBJECT$CLASS::run(Scope* scope) {
    if (!classRef) return nullptr;

    NODE* inst = scope->instance;
    std::vector<NODE*> instances;
    if (inst) instances.push_back(inst);
    else for (auto& [k, v] : classRef->instances) if (v) instances.push_back(v);

    for (NODE* i : instances) {
        std::string instName = i->key;
        std::string id       = instName + "." + name;

        auto* oi = new OBJECT$INSTANCE(id);
        oi->classNode  = nullptr;
        oi->objectNode = Graph::instance().retrieve(instName);
        oi->name       = name;

        Scope* s  = new Scope(scope);
        s->instance = i;

        return oi; // Stack handles iteration for subsequent instances
    }
    return nullptr;
}

} // namespace nucleoid
