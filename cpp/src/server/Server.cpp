#include <httplib.h>
#include "nucleoid/server/Server.hpp"
#include "nucleoid/Nucleoid.hpp"
#include "nucleoid/Datastore.hpp"
#include "nucleoid/Graph.hpp"
#include "nucleoid/nuc/NODE.hpp"
#include "nucleoid/Types.hpp"
#include <nlohmann/json.hpp>
#include <iostream>
#include <stdexcept>
#include <string>

namespace nucleoid {

static void addCors(httplib::Response& res) {
    res.set_header("Access-Control-Allow-Origin",  "*");
    res.set_header("Access-Control-Allow-Headers", "Content-Type");
    res.set_header("Access-Control-Allow-Methods", "GET,POST,PUT,DELETE,OPTIONS");
}

static nlohmann::json bodyJson(const httplib::Request& req) {
    if (req.body.empty()) return {};
    try { return nlohmann::json::parse(req.body); } catch (...) { return {}; }
}

void serverStart(const Config& cfg) {
    int port = cfg.port.terminal > 0 ? cfg.port.terminal : 8448;

    httplib::Server svr;

    // ── OPTIONS pre-flight ────────────────────────────────────────────────────
    svr.Options(".*", [](const httplib::Request&, httplib::Response& res) {
        addCors(res);
        res.status = 204;
    });

    // ── POST /terminal  (run a statement) ────────────────────────────────────
    svr.Post("/terminal", [](const httplib::Request& req, httplib::Response& res) {
        addCors(res);
        auto body = bodyJson(req);
        std::string stmt = body.value("statement", "");
        bool details = body.value("details", false);

        Options opts;
        opts.details = details;

        try {
            nlohmann::json result = run(stmt, opts);
            nlohmann::json resp;
            resp["result"] = result;
            res.set_content(resp.dump(), "application/json");
        } catch (const std::exception& e) {
            nlohmann::json err;
            err["error"]   = true;
            err["message"] = e.what();
            res.status = 400;
            res.set_content(err.dump(), "application/json");
        }
    });

    // ── GET /terminal  (list log entries) ─────────────────────────────────────
    svr.Get("/terminal", [](const httplib::Request&, httplib::Response& res) {
        addCors(res);
        auto all = datastoreAll();
        nlohmann::json arr = nlohmann::json::array();
        for (auto& d : all) {
            arr.push_back({
                {"string",      d.string},
                {"declarative", d.declarative},
                {"time",        d.time},
                {"date",        d.date},
                {"error",       d.error}
            });
        }
        res.set_content(arr.dump(), "application/json");
    });

    // ── GET /graph  (knowledge graph) ─────────────────────────────────────────
    svr.Get("/graph", [](const httplib::Request&, httplib::Response& res) {
        addCors(res);
        nlohmann::json g = nlohmann::json::object();
        for (auto& [key, node] : Graph::instance().all()) {
            if (!node) { g[key] = nullptr; continue; }
            nlohmann::json n;
            n["key"]  = node->key;
            n["type"] = node->type;
            nlohmann::json nexts = nlohmann::json::array();
            for (auto& [k, _] : node->next) nexts.push_back(k);
            n["next"] = nexts;
            g[key] = n;
        }
        res.set_content(g.dump(), "application/json");
    });

    // ── GET /logs ─────────────────────────────────────────────────────────────
    svr.Get("/logs", [](const httplib::Request&, httplib::Response& res) {
        addCors(res);
        auto all = datastoreAll();
        nlohmann::json arr = nlohmann::json::array();
        for (auto& d : all) {
            arr.push_back({
                {"string", d.string},
                {"time",   d.time},
                {"date",   d.date},
                {"error",  d.error}
            });
        }
        res.set_content(arr.dump(), "application/json");
    });

    // ── GET /metrics ──────────────────────────────────────────────────────────
    svr.Get("/metrics", [](const httplib::Request&, httplib::Response& res) {
        addCors(res);
        auto all = datastoreAll();
        long long total = 0;
        for (auto& d : all) total += d.time;
        nlohmann::json m;
        m["count"]   = all.size();
        m["totalMs"] = total;
        m["avgMs"]   = all.empty() ? 0 : total / (long long)all.size();
        res.set_content(m.dump(), "application/json");
    });

    // ── 404 ───────────────────────────────────────────────────────────────────
    svr.set_error_handler([](const httplib::Request&, httplib::Response& res) {
        addCors(res);
        res.set_content(R"({"error":"Not found"})", "application/json");
    });

    std::cout << "Nucleoid terminal listening on port " << port << "\n";
    svr.listen("0.0.0.0", port);
}

} // namespace nucleoid
