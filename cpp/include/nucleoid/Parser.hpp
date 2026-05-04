#pragma once
#include "nuc/NODE.hpp"
#include <string>
#include <vector>

namespace nucleoid {

// Parses a JavaScript source string using QuickJS's built-in parser and
// converts each top-level statement into a Nucleoid NODE.
// Throws JSException on syntax errors.
std::vector<NODE*> parseStatements(const std::string& source,
                                   bool declarative = false);

} // namespace nucleoid
