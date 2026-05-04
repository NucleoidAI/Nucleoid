#include "nucleoid/JSEngine.hpp"
#include <cassert>
#include <cstring>
#include <sstream>

namespace nucleoid {

JSEngine& JSEngine::instance() {
    static JSEngine eng;
    return eng;
}

JSEngine::JSEngine() {
    rt_  = JS_NewRuntime();
    ctx_ = JS_NewContext(rt_);

    // Bootstrap: create the shared `state` object used by the Nucleoid runtime.
    // Use globalThis property assignment (not var) so the names are always
    // accessible as globals regardless of QuickJS eval-context semantics.
    const char* init =
        "globalThis.state = { classes: [] };"
        "globalThis.$     = { classes: {} };";
    JSValue initCode = JS_Eval(ctx_, init, (int)strlen(init),
                               "<init>", JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(initCode)) {
        JSValue ex = JS_GetException(ctx_);
        const char* s = JS_ToCString(ctx_, ex);
        std::string msg = s ? s : "init failed";
        JS_FreeCString(ctx_, s);
        JS_FreeValue(ctx_, ex);
        JS_FreeValue(ctx_, initCode);
        throw std::runtime_error("JSEngine init: " + msg);
    }
    JS_FreeValue(ctx_, initCode);
}

JSEngine::~JSEngine() {
    if (ctx_) JS_FreeContext(ctx_);
    if (rt_)  JS_FreeRuntime(rt_);
}

void JSEngine::reset() {
    if (ctx_) JS_FreeContext(ctx_);
    ctx_ = JS_NewContext(rt_);
    const char* init =
        "globalThis.state = { classes: [] };"
        "globalThis.$     = { classes: {} };";
    JSValue v = JS_Eval(ctx_, init, (int)strlen(init), "<init>", JS_EVAL_TYPE_GLOBAL);
    JS_FreeValue(ctx_, v);
}

// ── helpers ───────────────────────────────────────────────────────────────────

std::string JSEngine::exceptionString(JSValue ex) {
    std::string msg;
    if (JS_IsError(ctx_, ex)) {
        JSValue msgVal = JS_GetPropertyStr(ctx_, ex, "message");
        const char* s = JS_ToCString(ctx_, msgVal);
        if (s) { msg = s; JS_FreeCString(ctx_, s); }
        JS_FreeValue(ctx_, msgVal);

        JSValue stack = JS_GetPropertyStr(ctx_, ex, "stack");
        if (!JS_IsUndefined(stack)) {
            const char* ss = JS_ToCString(ctx_, stack);
            if (ss) { msg += "\n"; msg += ss; JS_FreeCString(ctx_, ss); }
        }
        JS_FreeValue(ctx_, stack);
    } else {
        const char* s = JS_ToCString(ctx_, ex);
        if (s) { msg = s; JS_FreeCString(ctx_, s); }
    }
    return msg;
}

void JSEngine::throwIfException(JSValue val, const std::string& ctx) {
    if (JS_IsException(val)) {
        JSValue ex = JS_GetException(ctx_);
        std::string msg = (ctx.empty() ? "" : ctx + ": ") + exceptionString(ex);
        JS_FreeValue(ctx_, ex);
        throw JSException(msg);
    }
}

// ── public API ────────────────────────────────────────────────────────────────

JSValue JSEngine::evalRaw(const std::string& code) {
    JSValue val = JS_Eval(ctx_, code.c_str(), code.size(),
                          "<eval>", JS_EVAL_TYPE_GLOBAL);
    throwIfException(val, "eval");
    return val;
}

nlohmann::json JSEngine::eval(const std::string& code) {
    JSValue val = evalRaw(code);
    nlohmann::json j = toJson(val);
    JS_FreeValue(ctx_, val);
    return j;
}

nlohmann::json JSEngine::get(const std::string& name) {
    JSValue global = JS_GetGlobalObject(ctx_);
    JSValue val    = JS_GetPropertyStr(ctx_, global, name.c_str());
    JS_FreeValue(ctx_, global);
    nlohmann::json j = toJson(val);
    JS_FreeValue(ctx_, val);
    return j;
}

void JSEngine::set(const std::string& name, const nlohmann::json& value) {
    JSValue global = JS_GetGlobalObject(ctx_);
    JSValue val    = fromJson(value);
    JS_SetPropertyStr(ctx_, global, name.c_str(), val);
    JS_FreeValue(ctx_, global);
}

bool JSEngine::del(const std::string& name) {
    JSValue global = JS_GetGlobalObject(ctx_);
    JSAtom  atom   = JS_NewAtom(ctx_, name.c_str());
    int     ret    = JS_DeleteProperty(ctx_, global, atom, 0);
    JS_FreeAtom(ctx_, atom);
    JS_FreeValue(ctx_, global);
    return ret == 1;
}

// ── JSON <-> JSValue conversion ───────────────────────────────────────────────

nlohmann::json JSEngine::toJson(JSValue val) {
    if (JS_IsUndefined(val) || JS_IsUninitialized(val))
        return nullptr;
    if (JS_IsNull(val))
        return nullptr;
    if (JS_IsBool(val))
        return static_cast<bool>(JS_VALUE_GET_BOOL(val));
    if (JS_IsNumber(val)) {
        double d;
        JS_ToFloat64(ctx_, &d, val);
        if (d == static_cast<int64_t>(d))
            return static_cast<int64_t>(d);
        return d;
    }
    if (JS_IsString(val)) {
        const char* s = JS_ToCString(ctx_, val);
        std::string str(s ? s : "");
        JS_FreeCString(ctx_, s);
        return str;
    }
    if (JS_IsArray(ctx_, val)) {
        nlohmann::json arr = nlohmann::json::array();
        JSValue len = JS_GetPropertyStr(ctx_, val, "length");
        uint32_t n = 0;
        JS_ToUint32(ctx_, &n, len);
        JS_FreeValue(ctx_, len);
        for (uint32_t i = 0; i < n; ++i) {
            JSValue elem = JS_GetPropertyUint32(ctx_, val, i);
            arr.push_back(toJson(elem));
            JS_FreeValue(ctx_, elem);
        }
        return arr;
    }
    if (JS_IsObject(val)) {
        // Serialise via JSON.stringify to handle circular refs gracefully.
        JSValue stringify = JS_Eval(ctx_,
            "(function(v){ try { return JSON.stringify(v); } catch(e){ return null; } })",
            80, "<str>", JS_EVAL_TYPE_GLOBAL);
        JSValue result = JS_Call(ctx_, stringify, JS_UNDEFINED, 1, &val);
        JS_FreeValue(ctx_, stringify);
        if (JS_IsNull(result) || JS_IsException(result)) {
            JS_FreeValue(ctx_, result);
            return nullptr;
        }
        const char* s = JS_ToCString(ctx_, result);
        std::string str(s ? s : "null");
        JS_FreeCString(ctx_, s);
        JS_FreeValue(ctx_, result);
        try { return nlohmann::json::parse(str); } catch (...) { return nullptr; }
    }
    return nullptr;
}

JSValue JSEngine::fromJson(const nlohmann::json& j) {
    if (j.is_null())    return JS_NULL;
    if (j.is_boolean()) return JS_NewBool(ctx_, j.get<bool>());
    if (j.is_number_integer())
        return JS_NewInt64(ctx_, j.get<int64_t>());
    if (j.is_number())
        return JS_NewFloat64(ctx_, j.get<double>());
    if (j.is_string()) {
        std::string s = j.get<std::string>();
        return JS_NewString(ctx_, s.c_str());
    }
    // For arrays/objects use JSON.parse so QuickJS owns the structure.
    std::string serialized = j.dump();
    std::string code = "JSON.parse(" + nlohmann::json(serialized).dump() + ")";
    JSValue v = JS_Eval(ctx_, code.c_str(), code.size(), "<json>", JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(v)) { JS_FreeValue(ctx_, v); return JS_NULL; }
    return v;
}

} // namespace nucleoid
