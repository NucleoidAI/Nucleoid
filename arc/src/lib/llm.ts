import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as Matrix from "./Matrix";
import * as openrouter from "./openrouter";

export async function generate({
  puzzle_id,
  only_puzzle_dataset,
  messages = [],
  document = true,
}) {
  if (document) {
    messages.unshift({
      role: "system",
      content: `
        few_shot_examples:
        ${dataset.document(puzzle_id, only_puzzle_dataset)}
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
