#pragma once
#include <nlohmann/json.hpp>
#include <string>
#include <unordered_map>
#include <vector>

namespace nucleoid {

class NODE;

class Scope {
public:
    Scope* prior;
    Scope* root;
    NODE*  block    = nullptr;
    NODE*  instance = nullptr;

    // Variables declared in this scope level.
    std::unordered_map<std::string, nlohmann::json> local;

    // Graph presence flags: variable name -> true.
    std::unordered_map<std::string, bool> graphFlags;

    // Class-instance lookup.
    std::unordered_map<std::string, nlohmann::json> instances;

    // Optional object context (class name).
    std::string objectName;

    // Deferred callbacks (set up by block statements).
    std::vector<void*> callback;

    explicit Scope(Scope* prior = nullptr);

    // Walk the scope chain to find the first scope that has `name` in its
    // graphFlags. Returns nullptr if not found.
    Scope* findOwner(const std::string& name) const;

    // Assign a value to `name` in this scope's local map and update graphFlags.
    void assign(const std::string& name, const nlohmann::json& value);

    // Retrieve the JSON value for `name` by walking the scope chain.
    // Returns nullptr JSON if not found.
    nlohmann::json retrieve(const std::string& name) const;

    // Retrieve the nearest scope instance (walks prior chain).
    NODE* getInstance() const;

    // Retrieve the nearest object name.
    std::string retrieveObject() const;

    // Look up an instance by name in the scope chain.
    nlohmann::json retrieveInstance(const std::string& name) const;
};

} // namespace nucleoid
