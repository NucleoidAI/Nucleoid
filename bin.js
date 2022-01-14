#!/usr/bin/env node

require("yargs")
  .scriptName("nucleoid")
  .command({
    command: "start",
    desc: "Start Nucleoid runtime",
    builder: (yargs) =>
      yargs
        .option("clean", {
          describe: "Clear data before starting the runtime",
        })
        .option("cache-only", {
          describe: "Start runtime without persistence unit",
        }),
    handler: (argv) => {
      if (argv.clear) {
        require("./src/libs/data").clear();
      }

      require("./server");
      console.log("Nucleoid runtime started");
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
