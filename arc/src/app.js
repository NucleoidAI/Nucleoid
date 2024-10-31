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

require.extensions[".md"] = (module, filename) => {
  const fs = require("fs");
  module.exports = fs.readFileSync(filename, "utf8").trim();
};

// ---

const analyzer = require("./lib/analyzer");
const visualizer = require("./lib/visualizer");
const nucleoid = require("./lib/nucleoid");
const Matrix = require("./lib/Matrix");
const { v4: uuid } = require("uuid");
const debug = require("./debug"); // eslint-disable-line no-unused-vars

const {
  train,
  test: [{ input: test_input_matrix, output: test_output_matrix }],
} = require("./data/training/3aa6fb7a.json"); // 0ca9ddb6.json

const train_dataset = {
  dataset: train.map(({ input, output }) => ({
    input_matrix: input,
    output_matrix: output,
    instances: [],
  })),
};

// const train_dataset = require("./debug")._3aa6fb7a.analyzer;

async function start() {
  const train_session_id = uuid();

  const { declarations } = await analyzer.declarations({
    train_dataset,
  });
  train_dataset.declarations = declarations;

  console.log("Creating declarations in Nucleoid...");
  await nucleoid.run(train_session_id, declarations.join("\n"));

  for (const dataset of train_dataset.dataset.reverse()) {
    const { input_matrix, output_matrix } = dataset;

    const { instances } = await analyzer.instances({
      declarations,
      input_matrix,
      output_matrix,
    });

    for (const {
      input_instance,
      output_instance,
      input_object,
      output_object,
    } of instances) {
      const instance_name = `obj${dataset.instances.length}`;

      const { input_code, output_value } = await analyzer.value({
        train_session_id,
        instance_name,
        declarations,
        input_object,
        output_object,
      });

      dataset.instances.push({
        instance_name,
        input_instance,
        output_instance,
        input_object,
        output_object,
        input_code,
        output_value,
      });
    }
  }

  /* Visualizing */

  // console.log("Waiting for 60 seconds due to rate limits...");
  // await new Promise((resolve) => setTimeout(resolve, 60 * 1000));

  const { instances } = await visualizer.instances({
    train_dataset,
    test_input_matrix,
  });

  // const instances = require("./debug")._3aa6fb7a.visualizer;

  const test_session_id = uuid();
  let test_index = 0;

  console.log("Initializing Nucleoid session with declarations...");
  await nucleoid.run(test_session_id, train_dataset.declarations.join("\n"));

  let result_matrix = Array.from({ length: test_output_matrix.length }, () =>
    Array(test_output_matrix[0].length).fill(0)
  );

  for (const { input_object } of instances.reverse()) {
    const { output_value } = await visualizer.value({
      instance_name: `obj${test_index++}`,
      test_session_id,
      train_dataset,
      input_object,
    });

    const { output_instance } = await visualizer.output_instance({
      test_input_matrix,
      result_matrix,
      train_dataset,
      input_object,
      output_value,
    });

    result_matrix = Matrix.merge(result_matrix, output_instance);
  }

  console.debug("Result:");
  Matrix.toString(result_matrix);
  console.debug("Expected:");
  Matrix.toString(test_output_matrix);
  return result_matrix;
}

module.exports = start;
