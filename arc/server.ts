import "dotenv/config";
import { start } from "./src/app";

start().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
