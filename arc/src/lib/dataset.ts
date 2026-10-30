import { readFile } from "node:fs/promises";
import { join } from "node:path";

async function read(puzzle_id: string) {
  return JSON.parse(
    await readFile(
      join(__dirname, "../..", "data", "training", `${puzzle_id}.json`),
      "utf8"
    )
  );
}

export default { read };
