#pragma once
extern "C" {
#include "third_party/quickjs.h"
}
#include <nlohmann/json.hpp>
#include <stdexcept>
#include <string>

namespace nucleoid {

class JSException : public std::runtime_error {
public:
    explicit JSException(const std::string& msg) : std::runtime_error(msg) {}
};

// Thread-local singleton wrapping one QuickJS runtime + context.
// All eval(), get(), set() calls operate on the same JS heap so that
// the "state" object is globally shared — matching the TS behaviour.
class JSEngine {
public:
    static JSEngine& instance();

    JSEngine(const JSEngine&)            = delete;
    JSEngine& operator=(const JSEngine&) = delete;

    // Evaluate a JS expression/statement; returns the result as JSON.
    nlohmann::json eval(const std::string& code);

    // Evaluate and return raw JSValue (caller must free with JS_FreeValue).
    JSValue evalRaw(const std::string& code);

    // Read a global variable as JSON.
    nlohmann::json get(const std::string& name);

    // Set a global variable from JSON.
    void set(const std::string& name, const nlohmann::json& value);

    // Delete a global variable; returns true on success.
    bool del(const std::string& name);

    // Convert a JSValue to JSON (JS_FreeValue NOT called).
    nlohmann::json toJson(JSValue val);

    // Convert JSON to a new JSValue (caller owns it).
    JSValue fromJson(const nlohmann::json& j);

    JSContext* ctx() const { return ctx_; }

    // Reset runtime state (used between tests).
    void reset();

private:
    JSEngine();
    ~JSEngine();

    void throwIfException(JSValue val, const std::string& context = "");
    std::string exceptionString(JSValue ex);

    JSRuntime* rt_  = nullptr;
    JSContext* ctx_ = nullptr;
};

} // namespace nucleoid
