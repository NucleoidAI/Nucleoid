import chalk from "chalk";
import { Config, Data, Options } from "./types";
import runtime from "./runtime";
import { init } from "./config";

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

export default { start, run };
