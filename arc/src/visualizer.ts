import * as nucleoid from "./lib/nucleoid";
import * as Matrix from "./lib/Matrix";
import * as llm from "./lib/llm";
import * as milestone from "./lib/milestone.ts";

export async function instances({
  puzzle_id,
  only_puzzle_dataset,
  train_dataset,
  test_input_matrix,
}) {
  milestone.segmenting_test_input({ test_input_matrix });

  const { dataset } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    messages: [
      {
        role: "assistant",
        content: `
          current:
          ${JSON.stringify(train_dataset)}
        `,
      },
      {
        role: "user",
        content: `
          query:
          ${JSON.stringify({
            dataset: {
              input_matrix: test_input_matrix,
            },
          })}
          json_format:
          {
            dataset: [{
              "instances": [{
                "input_instance_matrix": <INPUT_INSTANCE_MATRIX>
              }]
            }]
          }
        `,
      },
    ],
  });

  const instances = dataset[0].instances;

  milestone.test_input_segmented({ instances });

  return { instances };
}

export async function value({
  puzzle_id,
  only_puzzle_dataset,
  train_dataset,
  test_input_matrix,
  instance_name,
  input_instance_matrix,
}) {
  milestone.synthesizing_instance({ instance_name });

  const instance = {
    instance_name,
    input_instance_matrix,
  };

  const { x, y, width, height } = Matrix.locate(input_instance_matrix);

  const { orientation } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    few_shot: (train_dataset) =>
      train_dataset.dataset
        .flat()
        .flatMap(({ instances }) => instances)
        .map(({ input_instance_matrix, input_value }) => ({
          matrix: Matrix.silhouette(Matrix.crop(input_instance_matrix)),
          orientation: input_value.orientation,
        })),
    messages: [
      {
        role: "assistant",
        content: `
          current:
          ${JSON.stringify(
            train_dataset.dataset
              .flat()
              .filter(({ instances }) => instances)
              .flatMap(({ instances }) => instances)
              .filter(({ input_value }) => input_value)
              .map(({ input_instance_matrix, input_value }) => ({
                matrix: Matrix.silhouette(Matrix.crop(input_instance_matrix)),
                orientation: input_value.orientation,
              }))
          )}
        `,
      },
      {
        role: "user",
        content: `
          query:
          ${JSON.stringify({
            matrix: Matrix.silhouette(
              Matrix.crop(instance.input_instance_matrix)
            ),
          })}
          json_format:
          {
            "orientation": <ORIENTATION>,
          }
        `,
      },
    ],
  });

  const { dataset } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    few_shot: (train_dataset) => ({
      ...train_dataset,
      declarations: train_dataset.declarations[0],
      dataset: train_dataset.dataset.map(({ instances }) => ({
        instances: instances.map((instance) => ({
          ...instance,
          input_matrix: Matrix.crop(instance.input_instance_matrix),
          output_matrix: Matrix.crop(instance.output_instance_matrix),
        })),
      })),
    }),
    messages: [
      {
        role: "assistant",
        content: `
          current:
          ${JSON.stringify({
            ...train_dataset,
            declarations: train_dataset.declarations[0],
            dataset: train_dataset.dataset
              .filter(({ instances }) => instances)
              .map(({ instances }) => ({
                instances: instances
                  .filter(({ implementation }) => implementation)
                  .map((instance) => ({
                    ...instance,
                    input_matrix: Matrix.crop(instance.input_instance_matrix),
                    output_matrix: Matrix.crop(instance.output_instance_matrix),
                  })),
              })),
          })}
        `,
      },
      {
        role: "user",
        content: `
          query:
          ${JSON.stringify({
            dataset: [
              {
                instances: [instance],
              },
            ],
          })}
          input_x: ${x}
          input_y: ${y}
          input_width: ${width}
          input_height: ${height}
          orientation: ${orientation}
          json_format:
          {
            "dataset": [{
              "instances": [{
                "implementation": <IMPLEMENTATION>
              }]
            }]
          }  
        `,
      },
    ],
  });

  const implementation = dataset[0].instances[0].implementation;

  milestone.instance_synthesized({
    instance_name,
    input_instance_matrix,
    implementation,
  });
  milestone.executing_transformation({ instance_name });

  const canvas_width: number = test_input_matrix[0].length;
  const canvas_height: number = test_input_matrix.length;
  const input_value = await nucleoid.run(
    [
      `canvas_width = ${canvas_width}`,
      `canvas_height = ${canvas_height}`,
      train_dataset.declarations[0],
      implementation,
    ].join("\n\n")
  );

  milestone.transformation_executed({ instance_name, input_value });

  return { implementation, input_value };
}

export async function output_instance_matrix({
  puzzle_id,
  only_puzzle_dataset,
  test_input_matrix,
  train_dataset,
  instance,
}) {
  const { x, y } = instance.output_value;

  milestone.composing_output({ instance_name: instance.instance_name });

  const { dataset } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    few_shot: (train_dataset) => ({
      ...train_dataset,
      dataset: train_dataset.dataset.map(({ instances }) => ({
        instances: instances.map((instance) => ({
          ...instance,
          input_matrix: Matrix.crop(instance.input_instance_matrix),
          output_matrix: Matrix.crop(instance.output_instance_matrix),
        })),
      })),
    }),
    messages: [
      {
        role: "assistant",
        content: `
          current:
          ${JSON.stringify({
            ...train_dataset,
            dataset: train_dataset.dataset.map(({ instances }) => ({
              instances: instances.map((instance) => ({
                ...instance,
                input_matrix: Matrix.crop(instance.input_instance_matrix),
                output_matrix: Matrix.crop(instance.output_instance_matrix),
              })),
            })),
          })}
        `,
      },
      {
        role: "user",
        content: `
          query:
          ${JSON.stringify({
            dataset: [
              {
                instances: [
                  {
                    ...instance,
                    input_instance_matrix: Matrix.crop(
                      instance.input_instance_matrix
                    ),
                  },
                ],
              },
            ],
          })}
          json_format:
          {
            "dataset": [{
              "instances": [{
                "output_instance_matrix": <OUTPUT_INSTANCE_MATRIX>
              }]
            }]
          }
        `,
      },
    ],
  });

  const output_instance_matrix = dataset[0].instances[0].output_instance_matrix;
  instance.output_instance_matrix = output_instance_matrix;

  milestone.output_composed(instance);

  const output_instance_padded_matrix = Matrix.pad(
    Matrix.crop(output_instance_matrix),
    test_input_matrix.length,
    test_input_matrix[0].length,
    x,
    y
  );

  milestone.output_placed(instance);

  return { output_instance_matrix: output_instance_padded_matrix };
}

export async function state({
  declarations,
  test_input_matrix,
  input_instances,
}) {
  const implementations = input_instances.map(
    ({ implementation }) => implementation
  );

  const canvas_width: number = test_input_matrix[0].length;
  const canvas_height: number = test_input_matrix.length;
  const obj_list = await nucleoid.run(
    [
      `canvas_width = ${canvas_width}`,
      `canvas_height = ${canvas_height}`,
      ...declarations,
      ...implementations,
      "Obj[:]",
    ].join("\n\n")
  );

  milestone.state({ phase: "visualization", obj_list });

  [...obj_list].forEach((instance) => {
    const found_instance = input_instances.find(
      ({ instance_name }) => instance.id === instance_name
    );
    found_instance.output_value = instance;
  });
}
