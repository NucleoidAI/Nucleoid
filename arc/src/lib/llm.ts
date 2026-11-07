import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as openrouter from "./openrouter";

export async function generate({
  puzzle_id,
  only_puzzle_dataset,
  few_shot = (instruction) => instruction,
  messages = [],
  document = true,
}) {
  if (document) {
    messages.unshift({
      role: "system",
      content: `
        few_shot_examples:
        ${examples(puzzle_id, only_puzzle_dataset, few_shot)}
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
  few_shot: (instruction) => unknown
): string {
  return JSON.stringify(
    JSON.parse(dataset.document(puzzle_id, only_puzzle_dataset)).map(few_shot)
  );
}
