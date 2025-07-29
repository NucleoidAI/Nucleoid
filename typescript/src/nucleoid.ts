import * as context from "./context";

import { Config, Data, Options } from "./types";

import chalk from "chalk";
import { init } from "./config";
import runtime from "./runtime";

function start(config: Config = {}) {
  init(config);

  console.log("🌿 " + chalk.greenBright("Nucleoid") + " runtime is started");
  console.log(chalk.blueBright("🌎 Inspired by Nature\n"));

  // process.init();
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function run(statement: string, options: Options = {}): Data | any {
  return runtime.process(statement, options);
}

function register(fn) {
  context.load([
    {
      definition: fn.toString(),
      options: { declarative: true },
    },
  ]);
}

export default { start, run, register };
