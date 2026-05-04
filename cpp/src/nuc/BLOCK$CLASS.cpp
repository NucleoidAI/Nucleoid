#include "nucleoid/nuc/BLOCK$CLASS.hpp"
#include "nucleoid/nuc/BLOCK$INSTANCE.hpp"
#include "nucleoid/Scope.hpp"
#include "nucleoid/Instruction.hpp"
#include <cstdint>
#include <random>
#include <sstream>

namespace nucleoid {

static std::string makeUUID() {
    static std::mt19937_64 rng(std::random_device{}());
    std::uniform_int_distribution<uint64_t> dist;
    std::ostringstream ss;
    ss << std::hex << dist(rng) << dist(rng);
    return ss.str();
}

NODE* BLOCK$CLASS::run(Scope* scope) {
    if (!classRef) return nullptr;

    std::vector<NODE*> toRun;

    // Determine which instances to iterate.
    std::vector<NODE*> instances;
    if (scope->instance) {
        instances.push_back(scope->instance);
    } else {
        for (auto& [k, v] : classRef->instances) {
            if (v) instances.push_back(v);
        }
    }

    for (NODE* inst : instances) {
        auto* blk = new BLOCK$INSTANCE(makeUUID());
        blk->instance    = inst;
        blk->declaration = this;
        blk->body        = statements;

        Scope* instanceScope = new Scope(scope);
        instanceScope->instance = inst;
        instanceScope->block    = blk;

        // We push the BLOCK$INSTANCE directly; Stack will expand it.
        toRun.push_back(blk);
    }

    // Return first; stack expansion handles the rest.
    return toRun.empty() ? nullptr : toRun.front();
}

} // namespace nucleoid
