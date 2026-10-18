import * as analyzer from "./analyzer";
import * as visualizer from "./visualizer";
import * as Matrix from "./lib/Matrix";
import puzzle from "./data/training/3aa6fb7a.json";
import type { Grid, TrainingDataset, TrainingInput } from "./types";

console.info = (message?: unknown) => {
  let string = message;

  if (typeof message !== "string") {
    string = JSON.stringify(message);
  }

  if (message === undefined) {
    string = "undefined";
  }

  if (message === null) {
    string = "null";
  }

  process.stdout.write(`\x1b[34m${string}\x1b[0m\n`);
};

console.debug = (message?: unknown) => {
  let string = message;

  if (typeof message !== "string") {
    string = JSON.stringify(message);
  }
  if (message === undefined) {
    string = "undefined";
  }

  if (message === null) {
    string = "null";
  }

  process.stdout.write(`\x1b[2m${string}\x1b[0m\n`);
};

console.log("🌿 \x1b[32mNucleoid\x1b[0m system is started");
console.log("\x1b[34m🌎 Inspired by Nature\x1b[0m\n");

// ---

const {
  train,
  test: [{ input: test_input_matrix, output: test_output_matrix }],
} = puzzle; // 0ca9ddb6.json

const training_input: TrainingInput = {
  dataset: train.map(({ input, output }) => ({
    input_matrix: input,
    output_matrix: output,
    instances: [],
  })),
};

export async function start(): Promise<Grid> {
  const { declarations } = await analyzer.declarations({
    train_dataset: training_input,
  });
  const train_dataset: TrainingDataset = {
    ...training_input,
    declarations,
  };

  for (const dataset of train_dataset.dataset) {
    const { input_matrix, output_matrix } = dataset;

    const { instances } = await analyzer.instances({
      declarations,
      input_matrix,
      output_matrix,
    });

    for (const { input_instance_matrix, output_instance_matrix } of instances) {
      const instance_name = `obj${dataset.instances.length}`;

      const { input_code, input_value, output_value } = await analyzer.value({
        instance_name,
        declarations,
        input_instance_matrix,
        output_instance_matrix,
      });

      dataset.instances.push({
        instance_name,
        input_instance_matrix,
        output_instance_matrix,
        input_code,
        input_value,
        output_value,
      });
    }
  }

  /* Visualizing */

  const { instances } = await visualizer.instances({
    train_dataset,
    test_input_matrix,
  });

  let test_index = 0;

  const result_matrix: Grid = Array.from(
    { length: test_output_matrix.length },
    () => Array<number>(test_output_matrix[0].length).fill(0)
  );

  for (const { input_instance_matrix } of instances) {
    const { input_value, output_value } = await visualizer.value({
      instance_name: `obj${test_index++}`,
      train_dataset,
      input_instance_matrix,
    });

    const { output_instance_matrix } = await visualizer.output_instance_matrix({
      test_input_matrix,
      result_matrix,
      train_dataset,
      input_instance_matrix,
      input_value,
      output_value,
    });

    result_matrix = Matrix.merge(result_matrix, output_instance_matrix);
  }

  console.debug("");
  console.debug("Result:");
  Matrix.toString(result_matrix);
  console.debug("Expected:");
  Matrix.toString(test_output_matrix);
  console.log(
    "Difference:",
    Matrix.difference(result_matrix, test_output_matrix)
  );
  return result_matrix;
}
