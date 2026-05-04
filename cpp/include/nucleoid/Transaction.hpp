#pragma once
#include <nlohmann/json.hpp>
#include <string>
#include <variant>
#include <vector>

namespace nucleoid {

// Represents one mutation that can be rolled back.
struct VariableTx {
    std::string    variable; // JS path like "state.x"
    nlohmann::json before;
};

using Transaction = VariableTx;

void transactionStart();
std::vector<Transaction> transactionEnd();

// Register a mutation; evaluates current value of `variable` in JS, then
// assigns `value`. Returns the new value as JSON.
nlohmann::json transactionRegister(const std::string& variable,
                                   const nlohmann::json& value);

void transactionRollback();

} // namespace nucleoid
