#include "nucleoid/nuc/BLOCK.hpp"

namespace nucleoid {

BLOCK::BLOCK(const std::string& key) : NODE(key) { type = "BLOCK"; }

void BLOCK::before(Scope* /*scope*/) {}
NODE* BLOCK::run(Scope* /*scope*/) { return nullptr; }
void BLOCK::graph(Scope* /*scope*/) {}
void BLOCK::after(Scope* /*scope*/) {}

void BLOCK::stage(const Instruction& instr) {
    staged_.push_back(instr);
}

} // namespace nucleoid
