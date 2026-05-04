#include "nucleoid/Nucleoid.hpp"
#include "nucleoid/State.hpp"
#include "nucleoid/server/Server.hpp"
#include <iostream>
#include <string>
#include <vector>

static void printHelp() {
    std::cout <<
        "Usage: nucleoid <command> [options]\n\n"
        "Commands:\n"
        "  start              Start the Nucleoid HTTP terminal server\n"
        "  run <statement>    Execute a single statement and print the result\n"
        "  clear              Clear the runtime state\n"
        "  help               Show this help\n\n"
        "Options:\n"
        "  --port <n>         Terminal server port (default 8448)\n"
        "  --declarative      Enable declarative mode\n";
}

int main(int argc, char* argv[]) {
    if (argc < 2) { printHelp(); return 0; }

    nucleoid::Config cfg;
    std::string command = argv[1];
    std::string statement;

    for (int i = 2; i < argc; ++i) {
        std::string arg = argv[i];
        if (arg == "--port" && i + 1 < argc) {
            cfg.port.terminal = std::stoi(argv[++i]);
        } else if (arg == "--declarative") {
            cfg.options.declarative = true;
        } else if (command == "run") {
            statement += (statement.empty() ? "" : " ") + arg;
        }
    }

    nucleoid::start(cfg);

    if (command == "start") {
        nucleoid::serverStart(cfg);
    } else if (command == "run") {
        if (statement.empty()) {
            std::cerr << "Error: no statement provided\n";
            return 1;
        }
        try {
            auto result = nucleoid::run(statement);
            std::cout << result.dump(2) << "\n";
        } catch (const std::exception& e) {
            std::cerr << "Error: " << e.what() << "\n";
            return 1;
        }
    } else if (command == "clear") {
        nucleoid::state::clear();
        std::cout << "State cleared.\n";
    } else if (command == "help") {
        printHelp();
    } else {
        std::cerr << "Unknown command: " << command << "\n";
        printHelp();
        return 1;
    }

    return 0;
}
