import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as openrouter from "./openrouter";

export async function generate({ puzzle_id, messages = [], document = true }) {
  if (document) {
    messages.unshift({
      role: "system",
      content: JSON.stringify({
        few_shot: JSON.parse(dataset.document(puzzle_id)),
      }),
    });
  }

  messages.unshift({
    role: "system",
    content: JSON.stringify({ control_id: uuid() }),
  });

  return openrouter.generate({
    messages,
  });
}
