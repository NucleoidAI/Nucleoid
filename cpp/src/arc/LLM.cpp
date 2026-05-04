#include <httplib.h>
#include "nucleoid/arc/LLM.hpp"
#include <cstdlib>
#include <stdexcept>

namespace nucleoid::arc {

// ── HTTP helper ───────────────────────────────────────────────────────────────
// Performs a synchronous POST using cpp-httplib.
nlohmann::json httpPost(const std::string& url,
                        const nlohmann::json& headersJson,
                        const nlohmann::json& body) {
    // Split url into host and path.
    std::string host, path;
    std::string stripped = url;
    bool https = false;

    if (stripped.rfind("https://", 0) == 0) {
        stripped = stripped.substr(8);
        https = true;
    } else if (stripped.rfind("http://", 0) == 0) {
        stripped = stripped.substr(7);
    }

    auto slash = stripped.find('/');
    if (slash == std::string::npos) {
        host = stripped; path = "/";
    } else {
        host = stripped.substr(0, slash);
        path = stripped.substr(slash);
    }

    httplib::Headers headers;
    headers.insert({"Content-Type", "application/json"});
    for (auto& [k, v] : headersJson.items()) {
        if (v.is_string()) headers.insert({k, v.get<std::string>()});
    }

    std::string bodyStr = body.dump();

    // cpp-httplib does not support TLS by default without OpenSSL.
    // For HTTPS we connect on port 443; for HTTP on port 80.
    httplib::Client cli(host, https ? 443 : 80);
    cli.set_connection_timeout(30);
    cli.set_read_timeout(120);

    auto res = cli.Post(path, headers, bodyStr, "application/json");

    if (!res)
        throw std::runtime_error("LLM HTTP request failed: " +
                                 httplib::to_string(res.error()));
    if (res->status < 200 || res->status >= 300)
        throw std::runtime_error("LLM HTTP error " + std::to_string(res->status) +
                                 ": " + res->body);

    return nlohmann::json::parse(res->body);
}

// ── Backend implementations ───────────────────────────────────────────────────

static nlohmann::json callClaude(const GenerateParams& p) {
    const char* key = std::getenv("CLAUDE_API_KEY");
    if (!key) throw std::runtime_error("CLAUDE_API_KEY not set");

    nlohmann::json msgs = nlohmann::json::array();
    for (auto& m : p.messages)
        msgs.push_back({{"role", m.role}, {"content", m.content}});

    nlohmann::json body = {
        {"model",       p.model.empty() ? "claude-sonnet-4-6" : p.model},
        {"messages",    msgs},
        {"temperature", p.temperature},
        {"max_tokens",  p.max_tokens}
    };

    nlohmann::json hdrs = {{"x-api-key", key}, {"anthropic-version", "2023-06-01"}};
    auto resp = httpPost("https://api.anthropic.com/v1/messages", hdrs, body);

    std::string text = resp["content"][0]["text"].get<std::string>();
    return nlohmann::json::parse(text);
}

static nlohmann::json callOpenAI(const GenerateParams& p) {
    const char* key = std::getenv("OPENAI_API_KEY");
    if (!key) throw std::runtime_error("OPENAI_API_KEY not set");

    nlohmann::json msgs = nlohmann::json::array();
    for (auto& m : p.messages)
        msgs.push_back({{"role", m.role}, {"content", m.content}});

    nlohmann::json body = {
        {"model",             p.model.empty() ? "gpt-4o-2024-08-06" : p.model},
        {"messages",          msgs},
        {"temperature",       p.temperature},
        {"max_tokens",        p.max_tokens},
        {"response_format",   {{"type", "json_object"}}}
    };

    nlohmann::json hdrs = {{"Authorization", std::string("Bearer ") + key}};
    auto resp = httpPost("https://api.openai.com/v1/chat/completions", hdrs, body);

    std::string content = resp["choices"][0]["message"]["content"].get<std::string>();
    return nlohmann::json::parse(content);
}

static nlohmann::json callGemini(const GenerateParams& p) {
    const char* key = std::getenv("GEMINI_API_KEY");
    if (!key) throw std::runtime_error("GEMINI_API_KEY not set");

    std::string model = p.model.empty() ? "gemini-1.5-pro" : p.model;

    nlohmann::json parts = nlohmann::json::array();
    for (auto& m : p.messages)
        parts.push_back({{"text", m.content}});

    nlohmann::json body = {
        {"contents", {{{"parts", parts}}}},
        {"generationConfig", {
            {"temperature",   p.temperature},
            {"maxOutputTokens", p.max_tokens}
        }}
    };

    std::string url = "https://generativelanguage.googleapis.com/v1beta/models/" +
                      model + ":generateContent?key=" + key;
    auto resp = httpPost(url, {}, body);

    std::string text = resp["candidates"][0]["content"]["parts"][0]["text"].get<std::string>();
    return nlohmann::json::parse(text);
}

static nlohmann::json callMistral(const GenerateParams& p) {
    const char* key = std::getenv("MISTRAL_API_KEY");
    if (!key) throw std::runtime_error("MISTRAL_API_KEY not set");

    nlohmann::json msgs = nlohmann::json::array();
    for (auto& m : p.messages)
        msgs.push_back({{"role", m.role}, {"content", m.content}});

    nlohmann::json body = {
        {"model",       p.model.empty() ? "mistral-large-latest" : p.model},
        {"messages",    msgs},
        {"temperature", p.temperature},
        {"max_tokens",  p.max_tokens}
    };

    nlohmann::json hdrs = {{"Authorization", std::string("Bearer ") + key}};
    auto resp = httpPost("https://api.mistral.ai/v1/chat/completions", hdrs, body);

    std::string content = resp["choices"][0]["message"]["content"].get<std::string>();
    return nlohmann::json::parse(content);
}

static nlohmann::json callLlama(const GenerateParams& p) {
    const char* endpoint = std::getenv("LLAMA_ENDPOINT");
    if (!endpoint) throw std::runtime_error("LLAMA_ENDPOINT not set");

    nlohmann::json msgs = nlohmann::json::array();
    for (auto& m : p.messages)
        msgs.push_back({{"role", m.role}, {"content", m.content}});

    nlohmann::json body = {
        {"model",       p.model.empty() ? "llama3" : p.model},
        {"messages",    msgs},
        {"temperature", p.temperature},
        {"max_tokens",  p.max_tokens}
    };

    auto resp = httpPost(std::string(endpoint) + "/v1/chat/completions", {}, body);
    std::string content = resp["choices"][0]["message"]["content"].get<std::string>();
    return nlohmann::json::parse(content);
}

static nlohmann::json callAzure(const GenerateParams& p) {
    const char* key      = std::getenv("AZURE_API_KEY");
    const char* endpoint = std::getenv("AZURE_ENDPOINT");
    if (!key || !endpoint) throw std::runtime_error("AZURE_API_KEY or AZURE_ENDPOINT not set");

    nlohmann::json msgs = nlohmann::json::array();
    for (auto& m : p.messages)
        msgs.push_back({{"role", m.role}, {"content", m.content}});

    nlohmann::json body = {
        {"messages",    msgs},
        {"temperature", p.temperature},
        {"max_tokens",  p.max_tokens}
    };

    nlohmann::json hdrs = {{"api-key", key}};
    auto resp = httpPost(std::string(endpoint), hdrs, body);

    std::string content = resp["choices"][0]["message"]["content"].get<std::string>();
    return nlohmann::json::parse(content);
}

// ── Dispatcher ────────────────────────────────────────────────────────────────
nlohmann::json generate(const GenerateParams& params) {
    const char* backend = std::getenv("LLM");
    std::string b = backend ? backend : "";

    if (b == "CLAUDE")  return callClaude(params);
    if (b == "OPENAI")  return callOpenAI(params);
    if (b == "GEMINI")  return callGemini(params);
    if (b == "MISTRAL") return callMistral(params);
    if (b == "LLAMA")   return callLlama(params);
    if (b == "AZURE")   return callAzure(params);

    throw std::runtime_error("LLM env var not set. Use: CLAUDE, OPENAI, GEMINI, MISTRAL, LLAMA, AZURE");
}

} // namespace nucleoid::arc
