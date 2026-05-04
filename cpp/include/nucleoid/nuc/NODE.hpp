#pragma once
#include <atomic>
#include <string>
#include <unordered_map>

namespace nucleoid {

class Scope;

class NODE {
public:
    std::string key;
    std::unordered_map<std::string, NODE*> next;
    std::unordered_map<std::string, NODE*> previous;
    int sequence;

    // Optional fields used by specific subtypes (kept here for flat layout).
    std::string type;
    bool        asg       = false;
    bool        skip      = false;
    bool        prepared  = false;
    bool        destroyed = false;
    NODE*       block     = nullptr;

    explicit NODE(const std::string& k);
    virtual ~NODE() = default;

    virtual void        before(Scope* scope) {}
    virtual NODE*       run(Scope* scope)    { return nullptr; }
    virtual void        beforeGraph(Scope* scope) {}
    virtual void        graph(Scope* scope)  {}
    virtual void        after(Scope* scope)  {}

    // Graph management (mirrors static methods on TS NODE).
    static void registerNode(const std::string& key, NODE* node);
    static void replaceNode(const std::string& srcKey, NODE* target);
    static void directNode(const std::string& srcKey,
                           const std::string& tgtKey, NODE* target);

private:
    static std::atomic<int> s_sequence;
};

} // namespace nucleoid
