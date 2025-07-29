#!/usr/bin/env node

import { hideBin } from "yargs/helpers";
import yargs from "yargs";

yargs(hideBin(process.argv))
  .scriptName("nucleoid")
  .command({
    command: "start",
    describe: "Start Nucleoid runtime",
    builder: (yargs) =>
      yargs
        .option("id", {
          describe: "Set id for the runtime",
          type: "string",
        })
        .option("clear", {
          describe: "Clear data before starting the runtime",
          type: "boolean",
        })
        .option("silence", {
          describe: "Silence the console",
          type: "boolean",
        })
        .option("debug", {
          describe: "Enable debug mode",
          type: "boolean",
        })
        .option("cluster", {
          describe: "Enable cluster mode",
          type: "boolean",
        })
        .option("terminal-port", {
          describe: "Set terminal port number",
          type: "number",
        })
        .option("cluster-port", {
          describe: "Set cluster port number",
          type: "number",
        }),
    handler: async () => {
      await import("./server");
    },
  })
  .command({
    command: "clear",
    describe: "Clear data",
    builder: () => {},
    handler: async () => {
      const datastore = await import("./src/datastore");
      datastore.default.clear();
      console.log("Data is cleared");
    },
  })
  .demandCommand()
  .parse();
