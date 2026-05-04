#include "nucleoid/Runtime.hpp"
#include "nucleoid/Config.hpp"
#include "nucleoid/Datastore.hpp"
#include "nucleoid/Event.hpp"
#include "nucleoid/JSEngine.hpp"
#include "nucleoid/Parser.hpp"
#include "nucleoid/Stack.hpp"
#include "nucleoid/Transaction.hpp"
#include <chrono>
#include <ctime>
#include <iomanip>
#include <sstream>
#include <stdexcept>

namespace nucleoid {

static std::string nowISO() {
    auto now  = std::chrono::system_clock::now();
    auto time = std::chrono::system_clock::to_time_t(now);
    std::ostringstream ss;
    ss << std::put_time(std::gmtime(&time), "%Y-%m-%dT%H:%M:%SZ");
    return ss.str();
}

Data runtimeProcess(const std::string& source, Options options) {
    Config cfg = configGet();
    options.declarative = options.declarative || cfg.options.declarative;

    auto t0 = std::chrono::steady_clock::now();

    NucResult  result;
    bool       error = false;

    try {
        std::vector<NODE*> statements = parseStatements(source, options.declarative);

        if (statements.empty()) {
            return Data{ source, options.declarative, {}, 0, nowISO(), false, {} };
        }

        transactionStart();
        result = stackProcess(statements, nullptr, options);
        transactionEnd();
    } catch (const std::exception& ex) {
        transactionRollback();
        error = true;
        result.value = ex.what();
    }

    auto t1  = std::chrono::steady_clock::now();
    long long ms = std::chrono::duration_cast<std::chrono::milliseconds>(t1 - t0).count();

    auto events = eventList();
    eventClear();

    Data data {
        source,
        options.declarative,
        result,
        ms,
        nowISO(),
        error,
        events
    };

    datastoreWrite(data);

    if (error) throw std::runtime_error(result.value.get<std::string>());

    return data;
}

} // namespace nucleoid
