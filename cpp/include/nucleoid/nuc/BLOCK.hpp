#pragma once
#include "NODE.hpp"
#include "../Instruction.hpp"
#include <vector>

namespace nucleoid {

class Scope;

class BLOCK : public NODE {
public:
    std::vector<NODE*> body;
    bool               brk   = false; // set true when BREAK hits this block
    bool               skip_ = false;

    explicit BLOCK(const std::string& key);

    void  before(Scope* scope) override;
    NODE* run(Scope* scope) override;
    void  graph(Scope* scope) override;
    void  after(Scope* scope) override;

    // Called by BREAK to stage an instruction for later.
    void stage(const Instruction& instr);

private:
    std::vector<Instruction> staged_;
};

} // namespace nucleoid
