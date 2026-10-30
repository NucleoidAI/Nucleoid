import * as nucleoid from "./lib/nucleoid";
import * as Matrix from "./lib/Matrix";
import * as llm from "./lib/llm";
import * as milestone from "./milestone";

export async function instances({
  puzzle_id,
  train_dataset,
  declarations,
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
          ${JSON.stringify(declarations)}
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
            train_dataset.dataset.map(({ instances }) => ({
              instances: instances.map(
                ({ implementation, input_value, input_instance_matrix }) => ({
                  implementation,
                  input_value,
                  input_instance_matrix: Matrix.crop(input_instance_matrix),
                })
              ),
            }))
          )}
          input_instances:
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

  milestone.executing_transformation();
  const input_value = await nucleoid.run(
    [train_dataset.declarations[0], implementation].join("\n\n")
  );
  const output_value = await nucleoid.run(
    [...train_dataset.declarations, implementation].join("\n\n")
  );

  milestone.transformation_executed({ input_value, output_value });

  return { input_value, output_value };
}

export async function output_instance_matrix({
  puzzle_id,
  test_input_matrix,
  train_dataset,
  input_instances,
  input_instance_matrix,
  input_value,
  output_value,
}) {
  const { x, y } = output_value;

  milestone.composing_output();
  const { output_instance_matrix } = await llm.generate({
    puzzle_id,
    messages: [
      {
        role: "assistant",
        content: `
          train_dataset:
          ${JSON.stringify(
            train_dataset.dataset.map(({ instances }) => ({
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
          )}
          input_instances:
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

  milestone.output_composed({ output_instance_matrix });

  const output_instance_padded_matrix = Matrix.pad(
    output_instance_matrix,
    test_input_matrix.length,
    test_input_matrix[0].length,
    x,
    y
  );

  milestone.output_placed({ output_instance_padded_matrix });

  return { output_instance_matrix: output_instance_padded_matrix };
}
