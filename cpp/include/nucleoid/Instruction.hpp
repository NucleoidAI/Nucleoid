#pragma once

namespace nucleoid {

class Scope;
class NODE;

struct Instruction {
    Scope* scope      = nullptr;
    NODE*  statement  = nullptr;
    bool   before     = true;
    bool   run        = true;
    bool   graph      = false;
    bool   after      = false;
    bool   derivative = true;
    bool   priority   = false;

    Instruction() = default;
    Instruction(Scope* sc, NODE* st,
                bool bef, bool r, bool g, bool af,
                bool deriv = true, bool prio = false)
        : scope(sc), statement(st),
          before(bef), run(r), graph(g), after(af),
          derivative(deriv), priority(prio) {}
};

} // namespace nucleoid
