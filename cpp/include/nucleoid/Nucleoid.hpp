#pragma once
#include "Types.hpp"
#include <nlohmann/json.hpp>
#include <string>

namespace nucleoid {

// ── Public API (mirrors src/nucleoid.ts) ──────────────────────────────────────

// Initialise the runtime with optional configuration.
void   start(const Config& cfg = {});

// Run a JavaScript/Nucleoid statement and return the result value.
// If options.details is true, returns the full Data record serialised as JSON.
nlohmann::json run(const std::string& statement, Options options = {});

// Register a declarative function (its source is loaded as a context statement).
void registerFn(const std::string& fnSource);

// Convenience: register via a JSON object { "definition": "...", options: {...} }
void registerContext(const nlohmann::json& entry);

} // namespace nucleoid
