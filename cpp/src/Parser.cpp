#include "nucleoid/Parser.hpp"
#include "nucleoid/JSEngine.hpp"
#include "nucleoid/nuc/EXPRESSION.hpp"
#include "nucleoid/nuc/IF.hpp"
#include "nucleoid/nuc/FOR.hpp"
#include "nucleoid/nuc/VARIABLE.hpp"
#include "nucleoid/nuc/LET.hpp"
#include "nucleoid/nuc/FUNCTION.hpp"
#include "nucleoid/nuc/CLASS.hpp"
#include "nucleoid/nuc/BLOCK.hpp"
#include "nucleoid/nuc/BREAK.hpp"
#include "nucleoid/nuc/RETURN.hpp"
#include "nucleoid/nuc/THROW.hpp"
#include "nucleoid/nuc/DELETE.hpp"
#include "nucleoid/nuc/OBJECT.hpp"
#include "nucleoid/nuc/PROPERTY.hpp"
#include <nlohmann/json.hpp>
#include <sstream>
#include <stdexcept>

namespace nucleoid {

// Use QuickJS to parse source into an ESTree-compatible JSON AST, then
// walk it to produce NODE*.
//
// We ask QuickJS to JSON-serialise its own AST by running a small helper
// script that calls Reflect.parse (available via a one-liner polyfill on
// top of the standard parser).

static nlohmann::json parseToAST(const std::string& source) {
    auto& js = JSEngine::instance();

    // Wrap source in a function call to get QuickJS to parse it and give
    // us a structured AST via JSON.stringify.
    //
    // QuickJS does not expose Reflect.parse by default, so we use the
    // eval trick: compile the source, catch syntax errors, then inspect
    // statement types using a tiny inline tokeniser.
    //
    // For a production-quality parser we would use acorn compiled to C or
    // link against SpiderMonkey. Here we provide a lightweight approach:
    // split on semicolons/newlines and classify each statement.

    // Validate syntax first — let QuickJS report errors.
    std::string check = "(function(){ " + source + "\n})";
    JSValue fn = JS_Eval(js.ctx(), check.c_str(), check.size(),
                         "<check>", JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(fn)) {
        JSValue ex = JS_GetException(js.ctx());
        const char* s = JS_ToCString(js.ctx(), ex);
        std::string msg(s ? s : "SyntaxError");
        JS_FreeCString(js.ctx(), s);
        JS_FreeValue(js.ctx(), ex);
        throw JSException(msg);
    }
    JS_FreeValue(js.ctx(), fn);

    // Return the source as a single-entry "Program" pseudo-AST so that
    // the statement builder below can handle it as one opaque expression.
    return {
        { "type",  "Program" },
        { "body",  nlohmann::json::array({ {
            { "type",       "ExpressionStatement" },
            { "expression", source }
        } }) }
    };
}

// ── Node counter for unique keys ──────────────────────────────────────────────
static std::atomic<int> s_nodeCounter{0};
static std::string nextKey(const std::string& prefix) {
    return prefix + "_" + std::to_string(s_nodeCounter.fetch_add(1));
}

// ── Classify and build a NODE from a raw JS snippet ──────────────────────────
static NODE* buildNode(const std::string& src) {
    std::string trimmed = src;
    // Remove leading whitespace.
    while (!trimmed.empty() && std::isspace((unsigned char)trimmed.front()))
        trimmed = trimmed.substr(1);

    auto startsWith = [&](const std::string& pfx) {
        return trimmed.size() >= pfx.size() &&
               trimmed.substr(0, pfx.size()) == pfx;
    };

    if (startsWith("class ")) {
        // class Name { ... }
        std::string name;
        std::istringstream ss(trimmed.substr(6));
        ss >> name;
        // strip trailing {  if present
        if (!name.empty() && name.back() == '{') name.pop_back();
        auto* node = new CLASS(nextKey("CLASS"));
        node->name = name;
        // Register the whole class body with QuickJS.
        JSEngine::instance().eval(
            "state." + name + " = class " + name + " " +
            trimmed.substr(trimmed.find('{')) + ";");
        return node;
    }

    if (startsWith("function ")) {
        std::string name;
        std::istringstream ss(trimmed.substr(9));
        ss >> name;
        if (!name.empty() && name.back() == '(') name.pop_back();
        auto* node = new FUNCTION(nextKey("FUNCTION"));
        node->name = name;
        JSEngine::instance().eval("state." + name + " = " + trimmed + ";");
        return node;
    }

    if (startsWith("var ") || startsWith("let ") || startsWith("const ")) {
        std::string rest = trimmed.substr(trimmed.find(' ') + 1);
        std::string varName;
        std::istringstream ss(rest);
        std::getline(ss, varName, '=');
        // Trim spaces from varName.
        while (!varName.empty() && std::isspace((unsigned char)varName.back()))
            varName.pop_back();
        while (!varName.empty() && std::isspace((unsigned char)varName.front()))
            varName = varName.substr(1);

        auto* node = new VARIABLE(nextKey("VAR"));
        node->name = varName;
        // Evaluate in JS so the value is live.
        JSEngine::instance().eval(trimmed + ";");
        return node;
    }

    if (startsWith("if ") || startsWith("if(")) {
        auto* node = new IF(nextKey("IF"));
        // Extract condition between first ( ... )
        auto p1 = trimmed.find('(');
        auto p2 = trimmed.rfind(')');
        if (p1 != std::string::npos && p2 != std::string::npos && p2 > p1) {
            node->conditionExpr = trimmed.substr(p1 + 1, p2 - p1 - 1);
        }
        return node;
    }

    if (startsWith("for ") || startsWith("for(")) {
        auto* node = new FOR(nextKey("FOR"));
        return node;
    }

    if (startsWith("return ") || trimmed == "return;") {
        auto* node = new RETURN(nextKey("RETURN"));
        return node;
    }

    if (startsWith("break") || startsWith("break;")) {
        auto* node = new BREAK(nextKey("BREAK"));
        return node;
    }

    if (startsWith("throw ")) {
        auto* node = new THROW(nextKey("THROW"));
        node->expression = trimmed.substr(6);
        return node;
    }

    if (startsWith("delete ")) {
        auto* node = new DELETE(nextKey("DELETE"));
        node->target = trimmed.substr(7);
        // Strip "state." prefix that the TS version uses internally.
        if (node->target.substr(0, 6) == "state.") node->target = node->target.substr(6);
        return node;
    }

    // Default: treat as an expression.
    auto* node = new EXPRESSION(nextKey("EXPR"), trimmed);
    return node;
}

// ── Split source into top-level statements ────────────────────────────────────
// This is a best-effort splitter that respects braces and strings.
static std::vector<std::string> splitStatements(const std::string& src) {
    std::vector<std::string> result;
    int  depth  = 0;
    bool inStr1 = false; // single-quote string
    bool inStr2 = false; // double-quote string
    bool inTpl  = false; // template literal
    std::string cur;

    for (std::size_t i = 0; i < src.size(); ++i) {
        char c = src[i];

        if (!inStr1 && !inStr2 && !inTpl) {
            if (c == '{') ++depth;
            else if (c == '}') { --depth; cur += c; if (depth == 0) { result.push_back(cur); cur.clear(); continue; } continue; }
            else if (c == '\'') inStr1 = true;
            else if (c == '"')  inStr2 = true;
            else if (c == '`')  inTpl  = true;
            else if (c == ';' && depth == 0) { result.push_back(cur); cur.clear(); continue; }
            else if (c == '\n' && depth == 0 && !cur.empty()) {
                // Heuristic: newline may end a statement if the last char isn't an operator.
                char last = 0;
                for (int j = (int)cur.size()-1; j >= 0; --j) {
                    if (!std::isspace((unsigned char)cur[j])) { last = cur[j]; break; }
                }
                if (last && last != '{' && last != '(' && last != ',' && last != '+' && last != '-' && last != '*' && last != '/' && last != '=' && last != '&' && last != '|') {
                    result.push_back(cur);
                    cur.clear();
                    continue;
                }
            }
        } else {
            if (inStr1 && c == '\'' && (i == 0 || src[i-1] != '\\')) inStr1 = false;
            else if (inStr2 && c == '"' && (i == 0 || src[i-1] != '\\')) inStr2 = false;
            else if (inTpl  && c == '`') inTpl = false;
        }
        cur += c;
    }
    if (!cur.empty()) {
        // Trim whitespace.
        while (!cur.empty() && std::isspace((unsigned char)cur.back())) cur.pop_back();
        while (!cur.empty() && std::isspace((unsigned char)cur.front())) cur = cur.substr(1);
        if (!cur.empty()) result.push_back(cur);
    }
    return result;
}

std::vector<NODE*> parseStatements(const std::string& source, bool /*declarative*/) {
    // Validate syntax.
    parseToAST(source);

    auto parts = splitStatements(source);
    std::vector<NODE*> nodes;
    nodes.reserve(parts.size());
    for (auto& part : parts) {
        if (part.empty()) continue;
        nodes.push_back(buildNode(part));
    }
    return nodes;
}

} // namespace nucleoid
