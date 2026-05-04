#pragma once
#include "NODE.hpp"
#include "../Graph.hpp"
#include "../JSEngine.hpp"
#include "../State.hpp"
#include <nlohmann/json.hpp>
#include <string>
#include <unordered_map>
#include <vector>

namespace nucleoid {

class CLASS : public NODE {
public:
    std::string                                  name;
    std::vector<NODE*>                           methods;
    std::unordered_map<std::string, NODE*>       instances;
    std::unordered_map<std::string, nlohmann::json> declarations;

    explicit CLASS(const std::string& key) : NODE(key) { type = "CLASS"; }

    NODE* run(Scope* scope) override {
        // Check if an identical class definition already exists.
        NODE* existing = Graph::instance().retrieve(name);
        if (existing) {
            CLASS* cls = dynamic_cast<CLASS*>(existing);
            if (cls && cls->methods.size() == methods.size()) {
                destroyed = true;
                return nullptr;
            }
        }

        // Create the JS class skeleton.
        std::string code = "state." + name + " = class " + name + " {};";
        JSEngine::instance().eval(code);
        JSEngine::instance().eval("state.classes.push(state." + name + ");");
        return nullptr;
    }

    void beforeGraph(Scope* scope) override {
        if (destroyed) return;
        NODE* existing = Graph::instance().retrieve(key);
        if (CLASS* cls = dynamic_cast<CLASS*>(existing)) {
            declarations = cls->declarations;
        }
    }
};

} // namespace nucleoid
