import * as nucleoid from "./lib/nucleoid";
import * as Matrix from "./lib/Matrix";
import * as llm from "./lib/llm";
import * as milestone from "./lib/milestone.ts";

export async function instances({
  puzzle_id,
  train_dataset,
  test_input_matrix,
}) {
  milestone.segmenting_test_input({ test_input_matrix });

  const { instances } = await llm.generate({
    puzzle_id,
    messages: [
      {
        role: "assistant",
        content: `
          declarations:
          ${JSON.stringify(train_dataset.declarations)}
          train_dataset:
          ${JSON.stringify(
            train_dataset.dataset.map(
              ({ input_matrix, output_matrix, instances }) => ({
                input_matrix,
                output_matrix,
                instances: instances.map(
                  ({ input_instance_matrix, output_instance_matrix }) => ({
                    input_instance_matrix,
                    output_instance_matrix,
                  })
                ),
              })
            )
          )}
        `,
      },
      {
        role: "user",
        content: `
          input_matrix:
          ${JSON.stringify(test_input_matrix)}
          json_format:
          {
            "instances": [{
              "input_instance_matrix": <INPUT_INSTANCE_MATRIX>
            }]
          }
        `,
      },
    ],
  });

  milestone.test_input_segmented({ instances });

  return { instances };
}

export async function value({
  puzzle_id,
  declarations,
  train_dataset,
  test_input_matrix,
  instance_name,
  input_instances,
  input_instance_matrix,
}) {
  const { x, y } = Matrix.locate(input_instance_matrix);

  milestone.synthesizing_instance({ instance_name });

  const { implementation } = await llm.generate({
    puzzle_id,
    messages: [
      {
        role: "assistant",
        content: `
          declarations:
          ${JSON.stringify(declarations)}
          train_dataset:
          ${JSON.stringify(
            train_dataset.dataset
              .map(({ instances }) => ({
                instances: instances.map(
                  ({ implementation, input_value, input_instance_matrix }) => ({
                    implementation,
                    input_value,
                    input_instance_matrix: Matrix.crop(input_instance_matrix),
                  })
                ),
              }))
              .flatMap(({ instances }) => instances)
          )}
          instances:
          ${JSON.stringify(
            input_instances.map(({ input_value, input_instance_matrix }) => ({
              input_value,
              input_instance_matrix: Matrix.crop(input_instance_matrix),
            }))
          )}
        `,
      },
      {
        role: "user",
        content: `
          instance_name:
          ${instance_name}
          input_instance_matrix:
          ${JSON.stringify(Matrix.crop(input_instance_matrix))}
          input_x: ${x}
          input_y: ${y}
          json_format:
          {
            "orientation": <ORIENTATION>,
            "implementation": <IMPLEMENTATION>
          }
        `,
      },
    ],
  });

  milestone.instance_synthesized({ instance_name, implementation });
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
  instance_name,
  test_input_matrix,
  train_dataset,
  input_instances,
  input_instance_matrix,
  input_value,
  output_value,
}) {
  const { x, y } = output_value;

  milestone.composing_output({ instance_name });
  const { output_instance_matrix } = await llm.generate({
    puzzle_id,
    messages: [
      {
        role: "assistant",
        content: `
          train_dataset:
          ${JSON.stringify(
            train_dataset.dataset
              .map(({ instances }) => ({
                instances: instances.map(
                  ({
                    input_value,
                    input_instance_matrix,
                    output_value,
                    output_instance_matrix,
                  }) => ({
                    input_value,
                    input_instance_matrix: Matrix.crop(input_instance_matrix),
                    output_value,
                    output_instance_matrix: Matrix.crop(output_instance_matrix),
                  })
                ),
              }))
              .flatMap(({ instances }) => instances)
          )}
          instances:
          ${JSON.stringify(
            input_instances.map(
              ({
                input_value,
                input_instance_matrix,
                output_value,
                output_instance_matrix,
              }) => ({
                input_value,
                input_instance_matrix: Matrix.crop(input_instance_matrix),
                output_value,
                output_instance_matrix: Matrix.crop(output_instance_matrix),
              })
            )
          )}
        `,
      },
      {
        role: "user",
        content: `
          input_value:
          ${JSON.stringify(input_value)}
          input_instance_matrix:
          ${JSON.stringify(Matrix.crop(input_instance_matrix))}
          output_value:
          ${JSON.stringify(output_value)}
          json_format:
          {
            "output_instance_matrix": <OUTPUT_INSTANCE_MATRIX>
          }
        `,
      },
    ],
  });

  milestone.output_composed({ instance_name, output_instance_matrix });

  const output_instance_padded_matrix = Matrix.pad(
    Matrix.crop(output_instance_matrix),
    test_input_matrix.length,
    test_input_matrix[0].length,
    x,
    y
  );

  milestone.output_placed({ instance_name, output_instance_padded_matrix });

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
