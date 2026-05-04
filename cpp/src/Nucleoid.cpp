#include "nucleoid/Nucleoid.hpp"
#include "nucleoid/Config.hpp"
#include "nucleoid/Runtime.hpp"
#include <chrono>
#include <ctime>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdexcept>

namespace nucleoid {

void start(const Config& cfg) {
    configInit(cfg);
    std::cout << "Nucleoid runtime is started\n";
    std::cout << "Inspired by Nature\n";
}

nlohmann::json run(const std::string& statement, Options options) {
    Config cfg = configGet();
    options.declarative = options.declarative || cfg.options.declarative;

    Data data = runtimeProcess(statement, options);

    if (options.details) {
        nlohmann::json j;
        j["string"]      = data.string;
        j["declarative"] = data.declarative;
        j["value"]       = data.result.value;
        j["time"]        = data.time;
        j["date"]        = data.date;
        j["error"]       = data.error;
        j["events"]      = nlohmann::json::array();
        for (auto& e : data.events) {
            j["events"].push_back({ {"topic", e.topic}, {"data", e.data} });
        }
        return j;
    }

    return data.result.value;
}

void registerFn(const std::string& fnSource) {
    Options opts;
    opts.declarative = true;
    try {
        runtimeProcess(fnSource, opts);
    } catch (...) {}
}

void registerContext(const nlohmann::json& entry) {
    std::string def = entry.value("definition", "");
    if (def.empty()) return;
    Options opts;
    opts.declarative = entry.contains("options") &&
                       entry["options"].value("declarative", false);
    try {
        runtimeProcess(def, opts);
    } catch (...) {}
}

} // namespace nucleoid
