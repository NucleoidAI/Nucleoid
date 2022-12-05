#!/usr/bin/env node

const fs = require("fs");
require("yargs")
  .scriptName("nucleoid")
  .command({
    command: "start",
    desc: "Start Nucleoid runtime",
    builder: (yargs) =>
      yargs
        .option("id", {
          describe: "Set id for the runtime",
        })
        .option("clean", {
          describe: "Clear data before starting the runtime",
        })
        .option("cluster", {
          describe: "Enable cluster configuration",
        })
        .option("cache-only", {
          describe: "Start runtime without persistence unit",
        })
        .option("port", {
          describe: "Set port number",
        }),
    handler: (argv) => {
      if (argv.clear) {
        const path = `${require("os").homedir()}/.nuc/data`;
        fs.rmSync(path, { recursive: true, force: true });
        fs.mkdirSync(path, { recursive: true });
      }

      const nucleoid = require("./index");
      nucleoid.start();
      console.log("Nucleoid runtime started");

      if (argv.cluster) {
        const Process = require("./src/cluster/process");
        nucleoid.register(Process);
      }
    },
  })
  .command({
    command: "clear",
    desc: "Clear data",
    builder: () => {},
    handler: () => {
      require("./src/libs/data").clear();
      console.log("Data is cleared");
    },
  })
  .demandCommand()
  .argv.toString();
