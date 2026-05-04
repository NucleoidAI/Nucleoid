#pragma once
#include <nlohmann/json.hpp>
#include <string>
#include <vector>

namespace nucleoid {

class Scope;

// Mirrors src/state.ts — all mutations go through TransactionRegister so they
// can be rolled back.
namespace state {

// Assign `value` to `state.<variable>`.
nlohmann::json assign(Scope* scope, const std::string& variable,
                      const nlohmann::json& value);

// Call `state.<fn>(args…)`.
nlohmann::json call(Scope* scope, const std::string& fn,
                    const std::vector<nlohmann::json>& args = {});

// Evaluate an arbitrary JS expression string.
nlohmann::json expression(Scope* scope, const std::string& expr);

// Delete `state.<variable>`.
bool del(Scope* scope, const std::string& variable);

// Throw a JS exception (terminates by throwing a C++ exception).
[[noreturn]] void throwException(Scope* scope, const std::string& expr);

// Clear all state (between runs or tests).
void clear();

} // namespace state
} // namespace nucleoid
