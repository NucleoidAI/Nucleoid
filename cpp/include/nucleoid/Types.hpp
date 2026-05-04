#pragma once
#include <nlohmann/json.hpp>
#include <optional>
#include <string>
#include <vector>

namespace nucleoid {

struct Options {
    bool declarative = false;
    bool details     = false;
};

struct PortConfig {
    int terminal = 8448;
    int cluster  = 8449;
    int openapi  = 8450;
};

struct DataConfig {
    bool encryption = false;
};

struct Config {
    std::string   path;
    PortConfig    port;
    Options       options;
    bool          cache = false;
    DataConfig    data;
    std::string   id;
    bool          test = false;
};

struct Event {
    std::string topic;
    std::string data;
};

struct NucResult {
    nlohmann::json nuc   = nlohmann::json::array();
    nlohmann::json value = nullptr;
};

struct Data {
    std::string              string;
    bool                     declarative = false;
    NucResult                result;
    long long                time   = 0;
    std::string              date;
    bool                     error  = false;
    std::vector<Event>       events;
};

} // namespace nucleoid
