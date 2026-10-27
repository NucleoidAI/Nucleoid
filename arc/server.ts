import "dotenv/config";
import { start } from "./src/app";
import * as terminal from "./src/lib/terminal";

start(process.argv[2]).catch((error) => {
  terminal.failure(error);
  process.exitCode = 1;
});
