#include "nucleoid/Config.hpp"
#include <optional>

namespace nucleoid {

static std::optional<Config> g_config;

void configInit(const Config& cfg) {
    g_config = cfg;
}

Config configGet() {
    return g_config.value_or(Config{});
}

void configClear() {
    g_config.reset();
}

} // namespace nucleoid
