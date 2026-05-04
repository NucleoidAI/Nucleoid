#include "nucleoid/Event.hpp"

namespace nucleoid {

static std::vector<Event> g_events;

void eventEmit(const std::string& topic, const std::string& data) {
    g_events.push_back({ topic, data });
}

std::vector<Event> eventList() { return g_events; }

void eventClear() { g_events.clear(); }

} // namespace nucleoid
