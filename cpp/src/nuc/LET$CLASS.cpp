#include "nucleoid/nuc/LET$CLASS.hpp"
#include "nucleoid/nuc/LET$INSTANCE.hpp"
#include "nucleoid/Scope.hpp"
#include <cstdint>
#include <random>
#include <sstream>

namespace nucleoid {

static std::string letUUID() {
    static std::mt19937_64 rng(std::random_device{}());
    std::uniform_int_distribution<uint64_t> dist;
    std::ostringstream ss;
    ss << std::hex << dist(rng);
    return ss.str();
}

NODE* LET$CLASS::run(Scope* scope) {
    if (!classRef) return nullptr;

    NODE* inst = scope->instance;
    if (!inst) return nullptr;

    auto* letInst = new LET$INSTANCE(letUUID());
    letInst->name         = name;
    letInst->valueExpr    = valueExpr;
    letInst->instanceNode = inst;
    letInst->className    = classRef->name;

    return letInst;
}

} // namespace nucleoid
