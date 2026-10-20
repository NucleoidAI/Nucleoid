import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as openrouter from "./openrouter";
import type { GenerateOptions } from "../types";

export async function generate<T = unknown>({
  messages = [],
  document = true,
}: GenerateOptions): Promise<T> {
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
        few_shot_dataset:
        ${dataset.document()}
      `,
    });
  }

  return openrouter.generate<T>({
    messages,
  });
}
