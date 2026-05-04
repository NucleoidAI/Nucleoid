#include "nucleoid/Transaction.hpp"
#include "nucleoid/JSEngine.hpp"
#include <sstream>

namespace nucleoid {

static std::vector<Transaction> g_list;

void transactionStart() {
    g_list.clear();
}

std::vector<Transaction> transactionEnd() {
    auto result = std::move(g_list);
    g_list.clear();
    return result;
}

nlohmann::json transactionRegister(const std::string& variable,
                                   const nlohmann::json& value) {
    auto& js = JSEngine::instance();

    // Capture current value before mutation.
    nlohmann::json before;
    try { before = js.eval(variable); } catch (...) { before = nullptr; }

    g_list.push_back({ variable, before });

    // Apply new value: stringify via nlohmann so we get valid JS literal.
    std::string code = variable + " = " + value.dump();
    return js.eval(code);
}

void transactionRollback() {
    auto& js = JSEngine::instance();
    while (!g_list.empty()) {
        auto tx = std::move(g_list.back());
        g_list.pop_back();
        try {
            std::string code = tx.variable + " = " + tx.before.dump();
            JSValue v = js.evalRaw(code);
            JS_FreeValue(js.ctx(), v);
        } catch (...) {}
    }
}

} // namespace nucleoid
