import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as Matrix from "./Matrix";
import * as openrouter from "./openrouter";

export async function generate({
  puzzle_id,
  only_puzzle_dataset,
  crop_instances = false,
  messages = [],
  document = true,
}) {
  if (document) {
    messages.unshift({
      role: "system",
      content: `
        few_shot_examples:
        ${examples(puzzle_id, only_puzzle_dataset, crop_instances)}
      `,
    });
  }

  messages.unshift({
    role: "system",
    content: `
      control_id: ${uuid()}
    `,
  });

  return openrouter.generate({
    messages,
  });
}

function examples(
  puzzle_id: string,
  only_puzzle_dataset: boolean,
  crop_instances: boolean
): string {
  const document: string = dataset.document(puzzle_id, only_puzzle_dataset);
  if (!crop_instances) {
    return document;
  }

  return JSON.stringify(
    JSON.parse(document).map((instruction) => ({
      ...instruction,
      dataset: instruction.dataset.map((pair) => ({
        ...pair,
        instances: pair.instances.map((instance) => ({
          ...instance,
          input_instance_matrix: Matrix.crop(instance.input_instance_matrix),
          output_instance_matrix: Matrix.crop(instance.output_instance_matrix),
        })),
      })),
    }))
  );
}
