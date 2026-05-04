#pragma once
#include "Types.hpp"
#include <string>

namespace nucleoid {

// Mirrors runtime.ts process().
// Parses `source`, runs it through the stack, persists to datastore, and
// returns a Data record.  If options.details is false, the caller receives
// only result.value (via Data::result.value).
Data runtimeProcess(const std::string& source, Options options = {});

} // namespace nucleoid
