import * as analyzer from "./analyzer";
import * as visualizer from "./visualizer";
import * as Matrix from "./lib/Matrix";
import * as nucleoid from "./lib/nucleoid";
import * as milestone from "./milestone";
import dataset from "./lib/dataset";

// Genesis puzzle_id: 3aa6fb7a
export async function start(puzzle_id: string = "3aa6fb7a") {
  const {
    test,
    train,
    test: [{ input: test_input_matrix }],
  } = await dataset.read(puzzle_id);

  const training_input = {
    dataset: train.map(({ input, output }) => ({
      input_matrix: input,
      output_matrix: output,
      instances: [],
    })),
  };

  const startedAt: number = performance.now();

  await milestone.start({ train, test_input_matrix });

  milestone.analysis();

  let instance_index: number = 0;

  const { declarations, output_canvas } = await analyzer.declarations({
    puzzle_id,
    train_dataset: training_input,
  });
  const train_dataset = {
    ...training_input,
    declarations,
    output_canvas,
  };

  for (const [index, dataset] of train_dataset.dataset.entries()) {
    const { input_matrix, output_matrix } = dataset;
    milestone.training_pair({ index, total: train_dataset.dataset.length });

    const { instances } = await analyzer.instances({
      puzzle_id,
      declarations,
      input_matrix,
      output_matrix,
    });

    for (const { input_instance_matrix, output_instance_matrix } of instances) {
      const instance_name: string = `obj${String(instance_index++).padStart(
        12,
        "0"
      )}`;

      const { implementation, input_value, output_value } =
        await analyzer.value({
          puzzle_id,
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

  milestone.visualization();

  const { instances } = await visualizer.instances({
    puzzle_id,
    train_dataset,
    declarations,
    test_input_matrix,
  });

  let result_matrix = Array.from({ length: test_input_matrix.length }, () =>
    Array(test_input_matrix[0].length).fill(0)
  );

  const input_instances = [];

  for (const { input_instance_matrix } of instances) {
    const instance_name: string = `obj${String(instance_index++).padStart(
      12,
      "0"
    )}`;

    const { implementation, input_value } = await visualizer.value({
      puzzle_id,
      declarations,
      train_dataset,
      instance_name,
      input_instances,
      input_instance_matrix,
    });

    input_instances.push({
      instance_name,
      implementation,
      input_value,
      input_instance_matrix,
    });
  }

  const output_values = await nucleoid.run(
    [
      ...declarations,
      ...input_instances.map(({ implementation }) => implementation),
      `[${input_instances
        .map(({ instance_name }) => instance_name)
        .join(", ")}]`,
    ].join("\n\n")
  );

  const output_instances = [];

  for (const [
    index,
    { input_value, input_instance_matrix },
  ] of input_instances.entries()) {
    const output_value = output_values[index];

    const { output_instance_matrix } = await visualizer.output_instance_matrix({
      puzzle_id,
      test_input_matrix,
      train_dataset,
      input_instances: output_instances,
      input_instance_matrix,
      input_value,
      output_value,
    });

    output_instances.push({
      input_value,
      input_instance_matrix,
      output_value,
      output_instance_matrix,
    });

    result_matrix = Matrix.merge(result_matrix, output_instance_matrix);
  }

  result_matrix = Matrix.canvas(result_matrix, output_canvas);

  const test_output_matrix = test[0].output;
  milestone.comparison({ test_output_matrix, result_matrix });

  const difference: number = Matrix.difference(
    result_matrix,
    test_output_matrix
  );

  milestone.summary({ difference, duration: performance.now() - startedAt });

  return difference;
}
