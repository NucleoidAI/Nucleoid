#pragma once
#include "Types.hpp"
#include "Instruction.hpp"
#include <vector>

namespace nucleoid {

class Scope;
class NODE;

// Mirrors stack.ts process().
// Takes a flat list of NODE* statements, processes them through the
// instruction pipeline, and returns a NucResult.
NucResult stackProcess(const std::vector<NODE*>& statements,
                       Scope*                    prior,
                       Options                   options = {});

} // namespace nucleoid
