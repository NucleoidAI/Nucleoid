import * as llm from "./lib/llm.ts";

async function synthesize({ train_dataset }) {
  const { dataset } = await llm.generate({
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
          synthesize:
          ${JSON.stringify({
            declarations: train_dataset.declarations,
            output_canvas: train_dataset.output_canvas,
            dataset: [
              {
                input_max_matrix: "<INPUT_MAX_MATRIX>",
                output_max_matrix: "<OUTPUT_MAX_MATRIX>",
                instances: [
                  {
                    input_instance_matrix: "<INPUT_INSTANCE_MATRIX>",
                    output_instance_matrix: "<OUTPUT_INSTANCE_MATRIX>",
                  },
                ],
              },
            ],
          })}
          json_format:
          {
            dataset: [
              {
                input_max_matrix: "<INPUT_MAX_MATRIX>",
                output_max_matrix: "<OUTPUT_MAX_MATRIX>",
                instances: [
                  {
                    input_instance_matrix: "<INPUT_INSTANCE_MATRIX>",
                    output_instance_matrix: "<OUTPUT_INSTANCE_MATRIX>",
                  },
                ],
              },
            ],
          }
          `,
      },
    ],
  });

  return dataset;
}

export default { synthesize };
