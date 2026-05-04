#include "nucleoid/Cache.hpp"
#include <nlohmann/json.hpp>
#include <vector>

namespace nucleoid {

static bool             g_enabled = false;
static std::vector<std::string> g_cache;

void cacheInit(const Config& cfg) {
    g_enabled = cfg.cache;
}

void cacheWrite(const Data& data) {
    if (!g_enabled) return;
    nlohmann::json j;
    j["string"]      = data.string;
    j["declarative"] = data.declarative;
    j["time"]        = data.time;
    j["date"]        = data.date;
    j["error"]       = data.error;
    g_cache.push_back(j.dump());
}

std::vector<Data> cacheRead() {
    std::vector<Data> result;
    for (auto& s : g_cache) {
        auto j = nlohmann::json::parse(s);
        Data d;
        d.string      = j.value("string", "");
        d.declarative = j.value("declarative", false);
        d.time        = j.value("time", 0LL);
        d.date        = j.value("date", "");
        d.error       = j.value("error", false);
        result.push_back(d);
    }
    return result;
}

std::vector<Data> cacheTail(int n) {
    auto all = cacheRead();
    std::reverse(all.begin(), all.end());
    if ((int)all.size() > n) all.resize(n);
    return all;
}

void cacheClear() { g_cache.clear(); }

} // namespace nucleoid
