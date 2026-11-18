import * as llm from "./lib/llm.ts";

async function synthesize({ train_dataset }) {
  const { result } = await llm.generate({
    few_shot: null,
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
                input_matrix: "<INPUT_MATRIX>",
                output_matrix: "<OUTPUT_MATRIX>",
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
            "result": [
              {
                "input_matrix": "<INPUT_MATRIX>",
                "output_matrix": "<OUTPUT_MATRIX>",
                "instances": [
                  {
                    "input_instance_matrix": "<INPUT_INSTANCE_MATRIX>",
                    "output_instance_matrix": "<OUTPUT_INSTANCE_MATRIX>",
                  },
                ],
              },
            ]
          }
          `,
      },
    ],
  });

  return result;
}

export default { synthesize };
