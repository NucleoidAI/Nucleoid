#include "nucleoid/Serialize.hpp"
#include <sstream>

namespace nucleoid {

std::string serialize(const nlohmann::json& value,
                      const std::string&    source,
                      std::unordered_set<std::string>* seenPtr) {
    std::unordered_set<std::string> localSeen;
    if (!seenPtr) seenPtr = &localSeen;

    if (value.is_null())    return "null";
    if (value.is_boolean()) return value.get<bool>() ? "true" : "false";
    if (value.is_number()) {
        std::ostringstream ss;
        if (value.is_number_integer())
            ss << value.get<int64_t>();
        else
            ss << value.get<double>();
        return ss.str();
    }
    if (value.is_string()) {
        return value.dump(); // includes surrounding quotes
    }
    if (value.is_array()) {
        return value.dump(); // JSON arrays are valid JS arrays
    }
    if (value.is_object()) {
        // Check for circular-reference sentinel.
        if (value.contains("$ref")) {
            auto ref = value["$ref"];
            std::string id  = ref.value("id", "");
            std::string src = ref.value("source", "");
            return "{$ref:{id:'" + id + "',source:'" + src + "'}}";
        }

        // Track by id if present.
        if (value.contains("id")) {
            std::string id = value["id"].dump();
            if (seenPtr->count(id)) {
                return "{$ref:{id:" + id + ",source:'" + source + "'}}";
            }
            seenPtr->insert(id);
        }

        std::string out = "{";
        bool first = true;
        for (auto& [k, v] : value.items()) {
            if (!first) out += ",";
            first = false;
            out += k + ":" + serialize(v, source, seenPtr);
        }
        out += "}";
        return out;
    }
    return "undefined";
}

} // namespace nucleoid
