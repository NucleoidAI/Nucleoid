import "dotenv/config";
import { start } from "./src/app";

start().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
