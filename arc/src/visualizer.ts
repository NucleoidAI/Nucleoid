import * as Matrix from "./lib/Matrix";
import * as llm from "./lib/llm";
import * as milestone from "./lib/milestone.ts";
import object from "./object.ts";
import * as nucleoid from "./lib/nucleoid";

export async function instances({ train, test_input_matrix }) {
  milestone.segmenting_test_input({ test_input_matrix });

  const parts = Matrix.separate(test_input_matrix);
  const parted_instances = [];

  for (const part of parts) {
    milestone.test_input_segmented({
      instances: [{ input_instance_matrix: part }],
    });

    const {
      result: { instances },
    } = await llm.generate({
      reasoning: false,
      grammar: true,
      messages: [
        {
          role: "assistant",
          content: `
          current:
          ${JSON.stringify(train)}
        `,
        },
        {
          role: "user",
          content: `
          query:
          ${JSON.stringify({
            class_declaration: train.class_declaration,
            declarations: train.declarations,
            output_canvas: train.output_canvas,
            dataset: [
              {
                input_matrix: part,
                instances: [
                  {
                    input_instance_crop_matrix: "<INPUT_INSTANCE_CROP_MATRIX>",
                    input_instance_matrix: "<INPUT_INSTANCE_MATRIX>",
                  },
                ],
              },
            ],
          })}
          json_format:
          {
            "result": {
              "instances": [{
                "input_instance_crop_matrix": "<INSTANCE_CROP_MATRIX>",
                "input_instance_matrix": "<INSTANCE_MATRIX>"
              }]
            }
          }
        `,
        },
      ],
    });

    parted_instances.push(...instances);
  }

  milestone.test_input_segmented({ instances: parted_instances });

  return { instances: parted_instances };
}

export async function value({
  train,
  test_input_matrix,
  instance_name,
  input_instance_matrix,
}) {
  const instance = {
    instance_name,
    input_instance_matrix,
  };

  const { x, y, width, height, volume } = Matrix.measure(input_instance_matrix);

  const orientation = await object.orientation({ train, instance });

  const implementation = await object.implementation({
    few_shot: false,
    train,
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
      train.class_declaration,
      implementation,
    ].join("\n\n")
  );

  milestone.transformation_executed({ instance_name, input_value });

  return { implementation, input_value };
}

export async function output_instance_matrix({
  test_input_matrix,
  train,
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
            ...train,
            dataset: train.dataset.map(({ instances }) => ({
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
