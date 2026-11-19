import * as Matrix from "./lib/Matrix";
import * as llm from "./lib/llm";
import * as milestone from "./lib/milestone.ts";
import object from "./object.ts";
import * as nucleoid from "./lib/nucleoid";

export async function instances({ train_dataset, test_input_matrix }) {
  milestone.segmenting_test_input({ test_input_matrix });

  let {
    result: { instances },
  } = await llm.generate({
    few_shot: null,
    grammar: true,
    messages: [
      {
        role: "assistant",
        content: `
          current:
          ${JSON.stringify({
            class_declaration: train_dataset.class_declaration,
            dataset: train_dataset.dataset.map(
              ({ input_matrix, instances }) => ({
                input_matrix,
                instances: instances.map((instance) => ({
                  instance_matrix: instance.input_instance_matrix,
                  instance_crop_matrix: Matrix.crop(
                    instance.input_instance_matrix
                  ),
                  implementation: instance.implementation,
                  x: instance.input_value.x,
                  y: instance.input_value.y,
                  width: instance.input_value.width,
                  height: instance.input_value.height,
                  orientation: instance.input_value.orientation,
                  volume: instance.input_value.volume,
                  value: instance.input_value,
                })),
              })
            ),
          })}
        `,
      },
      {
        role: "user",
        content: `
          query:
          ${JSON.stringify({
            class_declaration: train_dataset.class_declaration,
            dataset: [
              {
                instance_matrix: test_input_matrix,
                instances: [
                  {
                    instance_crop_matrix: "<INPUT_INSTANCE_CROP_MATRIX>",
                    instance_matrix: "<INSTANCE_MATRIX>",
                  },
                ],
              },
            ],
          })}
          json_format:
          {
            "result": {
              "instances": [{
                "instance_crop_matrix": "<INPUT_INSTANCE_CROP_MATRIX>",
                "instance_matrix": "<INSTANCE_MATRIX>"
              }]
            }
          }
        `,
      },
    ],
  });

  instances = instances.map((instance) => ({
    ...instance,
    input_instance_matrix: instance.instance_matrix,
    instance_matrix: undefined,
  }));

  milestone.test_input_segmented({ instances });

  return { instances };
}

export async function value({
  train_dataset,
  test_input_matrix,
  instance_name,
  input_instance_matrix,
}) {
  const instance = {
    instance_name,
    input_instance_matrix,
  };

  const { x, y, width, height, volume } = Matrix.locate(input_instance_matrix);

  const orientation = await object.orientation({ train_dataset, instance });

  const implementation = await object.implementation({
    few_shot: false,
    train_dataset,
    instance,
    x,
    y,
    width,
    height,
    orientation,
    volume,
  });

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
      train_dataset.class_declaration,
      implementation,
    ].join("\n\n")
  );

  milestone.transformation_executed({ instance_name, input_value });

  return { implementation, input_value };
}

export async function output_instance_matrix({
  test_input_matrix,
  train_dataset,
  instance,
}) {
  const { x, y } = instance.output_value;

  milestone.composing_output({ instance_name: instance.instance_name });

  const { output_instance_matrix } = await llm.generate({
    few_shot: null,
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
                input_instance_matrix: Matrix.crop(
                  instance.input_instance_matrix
                ),
                output_instance_matrix: Matrix.crop(
                  instance.output_instance_matrix
                ),
                width: instance.input_value.width,
                height: instance.input_value.height,
                orientation: instance.input_value.orientation,
                volume: instance.input_value.volume,
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
                    width: instance.input_value.width,
                    height: instance.input_value.height,
                    orientation: instance.input_value.orientation,
                    volume: instance.input_value.volume,
                    output_instance_matrix: "<OUTPUT_INSTANCE_MATRIX>",
                  },
                ],
              },
            ],
          })}
          json_format:
          {
            "output_instance_matrix": <OUTPUT_INSTANCE_MATRIX>
          }
        `,
      },
    ],
  });

  instance.output_instance_matrix = output_instance_matrix;

  milestone.output_composed(instance);

  const output_instance_padded_matrix = Matrix.pad(
    Matrix.crop(output_instance_matrix),
    test_input_matrix.length,
    test_input_matrix[0].length,
    x,
    y
  );

  milestone.output_placed({ ...instance, output_instance_padded_matrix });

  return { output_instance_matrix: output_instance_padded_matrix };
}
