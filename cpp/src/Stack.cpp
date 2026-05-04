#include "nucleoid/Stack.hpp"
#include "nucleoid/Graph.hpp"
#include "nucleoid/Instruction.hpp"
#include "nucleoid/Scope.hpp"
#include "nucleoid/State.hpp"
#include "nucleoid/nuc/BLOCK.hpp"
#include "nucleoid/nuc/BREAK.hpp"
#include "nucleoid/nuc/EXPRESSION.hpp"
#include "nucleoid/nuc/IF.hpp"
#include "nucleoid/nuc/NODE.hpp"
#include "nucleoid/nuc/RETURN.hpp"
#include <algorithm>
#include <deque>
#include <stdexcept>
#include <vector>

namespace nucleoid {

NucResult stackProcess(const std::vector<NODE*>& statements,
                       Scope*                    prior,
                       Options                   options) {
    Scope root(prior);

    std::deque<Instruction> instructions;
    for (NODE* stmt : statements) {
        instructions.push_back(
            Instruction(&root, stmt, true, true, false, false, false));
    }

    NucResult result;
    std::vector<NODE*> dependencies;
    std::vector<Instruction> dependents;
    std::vector<Instruction> priorities;

    while (!instructions.empty()) {
        Instruction instr = instructions.front();
        instructions.pop_front();

        NODE* stmt = instr.statement;
        if (!stmt) continue;

        // ── RETURN ───────────────────────────────────────────────────────────
        if (RETURN* ret = dynamic_cast<RETURN*>(stmt)) {
            std::vector<NODE*> inner;
            if (ret->statement) inner.push_back(ret->statement);
            return stackProcess(inner, instr.scope, options);
        }

        // ── BREAK ────────────────────────────────────────────────────────────
        if (BREAK* brk = dynamic_cast<BREAK*>(stmt)) {
            NODE* targetBlock = brk->block;
            while (!instructions.empty() && targetBlock &&
                   instructions.front().scope->block == targetBlock) {
                instructions.pop_front();
            }
            if (targetBlock) targetBlock->skip = true;
            continue;
        }

        // ── EXPRESSION (standalone, no graph) ────────────────────────────────
        if (EXPRESSION* expr = dynamic_cast<EXPRESSION*>(stmt)) {
            Scope* sc = instr.scope;
            expr->before(sc);
            expr->run(sc, false, false);

            nlohmann::json val = expr->lastValue;

            if (val.is_string()) {
                std::string s = val.get<std::string>();
                if (s == "use declarative") options.declarative = true;
                if (s == "use imperative")  options.declarative = false;
            }

            if (instr.scope == &root && !instr.derivative) {
                result.value = val;
            }
            continue;
        }

        // ── Default: full NUC node lifecycle ────────────────────────────────
        if (instr.before) {
            stmt->before(instr.scope);
        }

        if (instr.run) {
            NODE* nextNode = stmt->run(instr.scope);

            if (instr.scope == &root && !instr.derivative) {
                // Collect scalar result where applicable.
                if (EXPRESSION* e = dynamic_cast<EXPRESSION*>(stmt)) {
                    result.value = e->lastValue;
                }
            }

            if (nextNode) {
                std::vector<NODE*> nexts = { nextNode };
                for (NODE* n : nexts) {
                    Instruction ni(instr.scope, n, true, true, true, true);
                    ni.before     = ni.before     || instr.before;
                    ni.run        = ni.run        || instr.run;
                    ni.graph      = ni.graph      || instr.graph;
                    ni.after      = ni.after      || instr.after;
                    ni.derivative = instr.derivative;
                    if (ni.priority)
                        priorities.push_back(ni);
                    else
                        instructions.push_front(ni);
                }
            }
        }

        // ── Graph phase ──────────────────────────────────────────────────────
        if (instr.graph) {
            stmt->beforeGraph(instr.scope);
            if (stmt->destroyed) continue;

            if (Graph::instance().has(stmt->key)) {
                NODE::replaceNode(stmt->key, stmt);
            } else {
                NODE::registerNode(stmt->key, stmt);
            }

            stmt->graph(instr.scope);

            if (options.declarative) {
                // Dependency validation: check no circular references.
                for (NODE* dep : dependencies) {
                    if (dep->previous.count(stmt->key))
                        throw std::runtime_error("Circular Dependency");
                }
            }

            // Queue dependents (nodes that depend on this one).
            for (auto& [k, n] : stmt->next) {
                (void)k;
                Scope* depScope = new Scope(nullptr);
                Instruction dep(depScope, n, false, true, false, false);
                dependents.push_back(dep);
                Instruction dep2(depScope, n, false, false, true, true);
                dependents.push_back(dep2);
            }

            // Flush at root scope boundary.
            if (!instr.scope->prior) {
                if (!stmt->skip) {
                    for (NODE* src : dependencies) {
                        NODE::directNode(src->key, stmt->key, stmt);
                    }
                }
                for (auto& d : dependents)  instructions.push_back(d);
                for (auto& p : priorities)  instructions.push_back(p);
                dependencies.clear();
                dependents.clear();
                priorities.clear();
            }
        }

        // ── After phase ──────────────────────────────────────────────────────
        if (instr.after) {
            stmt->after(instr.scope);
            if (!instr.derivative && !stmt->asg) {
                result.nuc.push_back(stmt->key);
            }
        }
    }

    return result;
}

} // namespace nucleoid
