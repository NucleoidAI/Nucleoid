import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as openrouter from "./openrouter";

export async function generate({ messages = [], document = true }) {
  messages.unshift({
    role: "assistant",
    content: `
      control_id:
      ${uuid()}
    `,
  });
  if (document) {
    messages.unshift({
      role: "assistant",
      content: `
        dataset:
        ${dataset.document()}
      `,
    });
  }

  return openrouter.generate({
    messages,
  });
}
