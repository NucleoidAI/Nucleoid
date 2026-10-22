import * as analyzer from "./analyzer";
import * as visualizer from "./visualizer";
import * as Matrix from "./lib/Matrix";
import puzzle from "./data/training/3aa6fb7a.json";

console.info = (message) => {
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

console.debug = (message) => {
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

const training_input = {
  dataset: train.map(({ input, output }) => ({
    input_matrix: input,
    output_matrix: output,
    instances: [],
  })),
};

export async function start() {
  const { declarations } = await analyzer.declarations({
    train_dataset: training_input,
  });
  const train_dataset = {
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

      const { implementation, input_value, output_value } =
        await analyzer.value({
          instance_name,
          declarations,
          input_instance_matrix,
          output_instance_matrix,
        });

      dataset.instances.push({
        instance_name,
        input_instance_matrix,
        output_instance_matrix,
        implementation,
        input_value,
        output_value,
      });
    }
  }

  /* Visualizing */

  const { instances } = await visualizer.instances({
    train_dataset,
    declarations,
    test_input_matrix,
  });

  let test_index = 0;

  let result_matrix = Array.from({ length: test_output_matrix.length }, () =>
    Array(test_output_matrix[0].length).fill(0)
  );

  const input_instances = [];

  for (const { input_instance_matrix } of instances) {
    const { input_value, output_value } = await visualizer.value({
      declarations,
      train_dataset,
      instance_name: `obj${test_index++}`,
      input_instances,
      input_instance_matrix,
    });

    const { output_instance_matrix } = await visualizer.output_instance_matrix({
      test_input_matrix,
      train_dataset,
      input_instances,
      input_instance_matrix,
      input_value,
      output_value,
    });

    input_instances.push({
      input_value,
      input_instance_matrix,
      output_value,
      output_instance_matrix,
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
