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

  const { instances } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    few_shot: () => "",
    messages: [
      {
        role: "assistant",
        content: `
          declarations:
          ${JSON.stringify(train_dataset.declarations)}
          train_dataset:
          ${JSON.stringify(train_dataset.dataset)}
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
  only_puzzle_dataset,
  declarations,
  train_dataset,
  test_input_matrix,
  instance_name,
  input_instance_matrix,
}) {
  milestone.synthesizing_instance({ instance_name });

  const { x, y } = Matrix.locate(input_instance_matrix);

  const { orientation } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    few_shot: () => "",
    messages: [
      {
        role: "assistant",
        content: `
          train_dataset:
          ${JSON.stringify(
            train_dataset.dataset.flatMap(({ instances }) =>
              instances.map(({ input_instance_matrix, input_value }) => ({
                matrix: Matrix.silhouette(Matrix.crop(input_instance_matrix)),
                orientation: input_value.orientation,
              }))
            )
          )}
        `,
      },
      {
        role: "user",
        content: `
          matrix:
          ${JSON.stringify(
            Matrix.silhouette(Matrix.crop(input_instance_matrix))
          )}
          input_x: ${x}
          input_y: ${y}
          json_format:
          {
            "orientation": <ORIENTATION>,
          }
        `,
      },
    ],
  });

  const { implementation } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    few_shot: () => "",
    messages: [
      {
        role: "assistant",
        content: `
          declarations:
          ${JSON.stringify(declarations[0])}
          train_dataset:
          ${JSON.stringify(train_dataset.dataset)}
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
          orientation: ${orientation}
          json_format:
          {
            "implementation": <IMPLEMENTATION>
          }
        `,
      },
    ],
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
  instance_name,
  test_input_matrix,
  train_dataset,
  input_instance_matrix,
  input_value,
  output_value,
}) {
  const { x, y } = output_value;

  milestone.composing_output({ instance_name });

  const { output_instance_matrix } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    few_shot: () => "",
    messages: [
      {
        role: "assistant",
        content: `
          train_dataset:
          ${JSON.stringify(train_dataset.dataset)}
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

  milestone.output_composed({
    instance_name,
    input_instance_matrix,
    output_instance_matrix,
    output_value,
  });

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
