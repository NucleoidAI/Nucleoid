import "dotenv/config";
import { start } from "./src/app";
import * as terminal from "./src/lib/terminal";

start(process.argv[2])
  .then((difference: number) => {
    // Exit statuses keep eight bits, so 256 differences would read as 0.
    process.exitCode = Math.min(difference, 255);
  })
  .catch((error) => {
    terminal.failure(error);
    process.exitCode = 1;
  });
