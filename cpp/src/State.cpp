#include "nucleoid/State.hpp"
#include "nucleoid/JSEngine.hpp"
#include "nucleoid/Transaction.hpp"
#include <sstream>
#include <stdexcept>

namespace nucleoid::state {

nlohmann::json assign(Scope* /*scope*/, const std::string& variable,
                      const nlohmann::json& value) {
    std::string path = "state." + variable;
    return transactionRegister(path, value);
}

nlohmann::json call(Scope* /*scope*/, const std::string& fn,
                    const std::vector<nlohmann::json>& args) {
    std::string code = "state." + fn + "(";
    for (std::size_t i = 0; i < args.size(); ++i) {
        if (i) code += ",";
        code += args[i].dump();
    }
    code += ")";
    return JSEngine::instance().eval(code);
}

nlohmann::json expression(Scope* /*scope*/, const std::string& expr) {
    return JSEngine::instance().eval("(" + expr + ")");
}

bool del(Scope* /*scope*/, const std::string& variable) {
    return JSEngine::instance().eval("delete state." + variable).get<bool>();
}

[[noreturn]] void throwException(Scope* /*scope*/, const std::string& expr) {
    std::string msg;
    try {
        JSEngine::instance().eval(expr); // will throw JSException
        msg = expr;
    } catch (const std::exception& e) {
        msg = e.what();
    }
    throw std::runtime_error(msg);
}

void clear() {
    JSEngine::instance().eval(
        "state = { classes: [] }; $ = { classes: {} };");
}

} // namespace nucleoid::state
