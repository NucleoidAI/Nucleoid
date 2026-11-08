import { readFileSync } from "node:fs";
import { join } from "node:path";
import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as openrouter from "./openrouter";

export async function generate({
  puzzle_id,
  only_puzzle_dataset,
  reasoning = false,
  few_shot = (instruction) => instruction,
  messages = [],
  document = true,
  grammar = false,
}) {
  if (document) {
    messages.unshift({
      role: "system",
      content: `
        few_shot:
        ${examples(puzzle_id, only_puzzle_dataset, few_shot)}
      `,
    });
  }

  if (grammar) {
    messages.unshift({
      role: "system",
      content: readFileSync(
        join(__dirname, "..", "..", "..", "docs", "grammar.md"),
        "utf8"
      ),
    });
  }

  messages.unshift({
    role: "system",
    content: `
      control_id: ${uuid()}
    `,
  });

  return openrouter.generate({
    reasoning,
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
