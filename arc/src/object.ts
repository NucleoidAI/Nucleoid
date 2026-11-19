import * as llm from "./lib/llm.ts";
import * as Matrix from "./lib/Matrix.ts";
import * as nucleoid from "./lib/nucleoid";
import * as milestone from "./lib/milestone.ts";

async function orientation({ train_dataset, instance }) {
  const { orientation } = await llm.generate({
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
            orientation: "<ORIENTATION>",
          })}
          json_format:
          {
            "orientation": <ORIENTATION>,
          }
        `,
      },
    ],
  });

  return orientation;
}

async function implementation({
  few_shot = true,
  train_dataset,
  instance,
  x,
  y,
  width,
  height,
  orientation,
  volume,
}) {
  const few_shot_function = (train_dataset) => ({
    ...train_dataset,
    declarations: undefined,
    dataset: train_dataset.dataset.map(({ instances }) => ({
      instances: instances.map((instance) => ({
        ...instance,
        input_instance_matrix: Matrix.crop(instance.input_instance_matrix),
        output_instance_matrix: Matrix.crop(instance.output_instance_matrix),
      })),
    })),
  });
  const { implementation } = await llm.generate({
    few_shot: few_shot && few_shot_function,
    grammar: true,
    messages: [
      {
        role: "assistant",
        content: `
          current:
          ${JSON.stringify({
            ...train_dataset,
            declarations: undefined,
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
                implementation: "<IMPLEMENTATION>",
              },
            ],
          })}
          input_x: ${x}
          input_y: ${y}
          input_width: ${width}
          input_height: ${height}
          orientation: ${orientation}
          input_volume: ${volume}
          json_format:
          {
            "implementation": <IMPLEMENTATION>
          }  
        `,
      },
    ],
  });

  return implementation;
}

async function state({
  class_declaration,
  declarations,
  input_matrix,
  input_instances,
}) {
  const implementations = input_instances.map(
    ({ implementation }) => implementation
  );

  const canvas_width: number = input_matrix[0].length;
  const canvas_height: number = input_matrix.length;
  const obj_list = await nucleoid.run(
    [
      `canvas_width = ${canvas_width}`,
      `canvas_height = ${canvas_height}`,
      class_declaration,
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

export default { orientation, implementation, state };
