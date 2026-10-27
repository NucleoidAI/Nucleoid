import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as openrouter from "./openrouter";

export async function generate({ messages = [], document = true }) {
  if (document) {
    messages.unshift({
      role: "assistant",
      content: `
        instruct_dataset:
        ${dataset.document()}
      `,
    });
  }
  messages.unshift({
    role: "assistant",
    content: `
      control_id:
      ${uuid()}
    `,
  });

  return openrouter.generate({
    messages,
  });
}
