#include "nucleoid/nuc/IF$CLASS.hpp"
#include "nucleoid/nuc/IF$INSTANCE.hpp"
#include "nucleoid/Scope.hpp"
#include <cstdint>
#include <random>
#include <sstream>

namespace nucleoid {

static std::string ifUUID() {
    static std::mt19937_64 rng(std::random_device{}());
    std::uniform_int_distribution<uint64_t> dist;
    std::ostringstream ss;
    ss << std::hex << dist(rng);
    return ss.str();
}

NODE* IF$CLASS::run(Scope* scope) {
    if (!classRef) return nullptr;

    std::vector<NODE*> instances;
    if (scope->instance) {
        instances.push_back(scope->instance);
    } else {
        for (auto& [k, v] : classRef->instances) {
            if (v) instances.push_back(v);
        }
    }

    for (NODE* inst : instances) {
        auto* ifInst = new IF$INSTANCE(ifUUID());
        ifInst->conditionExpr = conditionExpr;
        ifInst->trueBranch    = trueBranch;
        ifInst->falseBranch   = falseBranch;
        ifInst->instanceNode  = inst;

        Scope* s = new Scope(scope);
        s->instance = inst;
        // Only return first; production would queue all.
        return ifInst;
    }
    return nullptr;
}

} // namespace nucleoid
