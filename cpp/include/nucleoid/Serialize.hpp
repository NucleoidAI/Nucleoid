#pragma once
#include <nlohmann/json.hpp>
#include <string>
#include <unordered_set>

namespace nucleoid {

// Serialize a JSON value to a JavaScript source-code literal string.
// Handles circular-reference tracking via a set of seen object ids.
std::string serialize(const nlohmann::json& value,
                      const std::string&    source,
                      std::unordered_set<std::string>* seen = nullptr);

} // namespace nucleoid
