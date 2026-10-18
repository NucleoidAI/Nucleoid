import { v4 as uuid } from "uuid";
import * as instruct_dataset from "../dataset";
import * as openrouter from "./openrouter";
import type { GenerateOptions } from "../types";

export async function generate<T = unknown>({
  messages = [],
}: GenerateOptions): Promise<T> {
  messages.unshift({
    role: "user",
    content: `
      control_id:
      ${uuid()}
    `,
  });
  messages.unshift({
    role: "user",
    content: instruct_dataset.document(),
  });

  return openrouter.generate<T>({
    messages,
  });
}
