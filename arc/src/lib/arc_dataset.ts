import { readFile } from "node:fs/promises";
import { join } from "node:path";

async function load(puzzle_id: string) {
  const puzzle = JSON.parse(
    await readFile(
      join(__dirname, "../..", "data", "training", `${puzzle_id}.json`),
      "utf8"
    )
  );
  const {
    train,
    test: [{ input: test_input_matrix, output: test_output_matrix }],
  } = puzzle;

  return { arc_train: train, test_input_matrix, test_output_matrix };
}

export default { load };
