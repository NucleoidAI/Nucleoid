#!/usr/bin/env node
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
        .option("clear", {
          describe: "Clear data before starting the runtime",
        })
        .option("cluster", {
          describe: "Enable cluster mode",
        })
        .option("cache-only", {
          describe: "Start runtime without persistence unit",
        })
        .option("terminal-port", {
          describe: "Set terminal port number",
        })
        .option("cluster-port", {
          describe: "Set cluster port number",
        }),
    handler: () => require("./server"),
  })
  .command({
    command: "clear",
    desc: "Clear data",
    builder: () => {},
    handler: () => {
      const datastore = require("@nucleoidjs/datastore");
      datastore.clear();
      console.log("Data is cleared");
    },
  })
  .demandCommand()
  .argv.toString();
