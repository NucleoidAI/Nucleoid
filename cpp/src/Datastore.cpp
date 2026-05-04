#include "nucleoid/Datastore.hpp"

namespace nucleoid {

static std::vector<Data> g_store;

void datastoreWrite(const Data& data) {
    g_store.push_back(data);
}

std::vector<Data> datastoreAll() { return g_store; }

void datastoreClear() { g_store.clear(); }

} // namespace nucleoid
