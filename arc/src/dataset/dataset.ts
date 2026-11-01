import { readFileSync } from "node:fs";
import { join } from "node:path";

// The genesis puzzle learns from the shared core, curve, and line datasets;
// every other puzzle learns only from its own dataset.<puzzle_id>.json.
const genesis_puzzle_id: string = "3aa6fb7a";
const genesis_datasets = [
  "dataset.core.1.json",
  "dataset.core.2.json",
  "dataset.curve.1.json",
  "dataset.line.1.json",
];

export function document(puzzle_id: string): string {
  const files =
    puzzle_id === genesis_puzzle_id
      ? genesis_datasets
      : [...genesis_datasets, `dataset.${puzzle_id}.json`];

  const datasets = files.map((file: string) => {
    const source: string = readFileSync(join(__dirname, file), "utf8");
    return source.replace(/("(?:[^"\\]|\\.)*")|\s+/g, "$1");
  });

  return `[${datasets.join(",")}]`;
}
