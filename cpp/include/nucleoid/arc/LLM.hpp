#pragma once
#include <nlohmann/json.hpp>
#include <string>
#include <vector>

namespace nucleoid::arc {

struct Message {
    std::string role;
    std::string content;
};

struct GenerateParams {
    std::string           model;
    std::vector<Message>  messages;
    double                temperature = 0.0;
    int                   max_tokens  = 2048;
};

// Select LLM backend from environment variable LLM:
//   CLAUDE, OPENAI, GEMINI, MISTRAL, LLAMA, AZURE
nlohmann::json generate(const GenerateParams& params);

// HTTP helper (shared by all backends).
nlohmann::json httpPost(const std::string& url,
                        const nlohmann::json& headers,
                        const nlohmann::json& body);

} // namespace nucleoid::arc
