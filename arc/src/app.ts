import * as analyzer from "./analyzer";
import * as visualizer from "./visualizer";
import * as Matrix from "./lib/Matrix";
import * as nucleoid from "./lib/nucleoid";
import * as terminal from "./lib/terminal";
import puzzle from "./data/training/0ca9ddb6.json";
// Completed:
// 3aa6fb7a

const {
  train,
  test: [{ input: test_input_matrix, output: test_output_matrix }],
} = puzzle;

const training_input = {
  dataset: train.map(({ input, output }) => ({
    input_matrix: input,
    output_matrix: output,
    instances: [],
  })),
};

export async function start() {
  const startedAt = performance.now();
  terminal.header({ version: await nucleoid.version() });
  terminal.puzzle({ train, testInput: test_input_matrix });
  terminal.section(
    1,
    "Analyze",
    "Infer reusable objects and transformation rules"
  );

  let instance_index = 0;

  const { declarations } = await analyzer.declarations({
    train_dataset: training_input,
  });
  const train_dataset = {
    ...training_input,
    declarations,
  };

  for (const [index, dataset] of train_dataset.dataset.entries()) {
    const { input_matrix, output_matrix } = dataset;
    terminal.step(
      `Training pair ${index + 1} / ${train_dataset.dataset.length}`
    );

    const { instances } = await analyzer.instances({
      declarations,
      input_matrix,
      output_matrix,
    });

    for (const { input_instance_matrix, output_instance_matrix } of instances) {
      const instance_name = `obj${String(instance_index++).padStart(12, "0")}`;

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

  terminal.section(
    2,
    "Visualize",
    "Apply the learned world model to the test input"
  );

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
      instance_name: `obj${String(test_index++).padStart(12, "0")}`,
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

  terminal.section(3, "Compare", "Expected and predicted canvases");
  terminal.grid("Expected", test_output_matrix);
  terminal.grid("Result", result_matrix);
  terminal.summary(
    Matrix.difference(result_matrix, test_output_matrix),
    performance.now() - startedAt
  );
  return result_matrix;
}
