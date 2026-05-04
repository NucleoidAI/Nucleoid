#include "nucleoid/Context.hpp"
#include "nucleoid/Nucleoid.hpp"

namespace nucleoid {

static std::vector<ContextItem> g_context;

void contextLoad(const std::vector<ContextItem>& items) {
    g_context.insert(g_context.end(), items.begin(), items.end());
}

void contextRun() {
    for (auto& item : g_context) {
        Options opts;
        opts.declarative = item.declarative;
        try { run(item.definition, opts); } catch (...) {}
    }
    g_context.clear();
}

} // namespace nucleoid
