import "dotenv/config";
import { start } from "./src/app";
import * as terminal from "./src/lib/terminal";

start().catch((error) => {
  terminal.failure(error);
  process.exitCode = 1;
});
