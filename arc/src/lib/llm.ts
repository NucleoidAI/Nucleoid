import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as openrouter from "./openrouter";

export async function generate({
  messages = [],
  response_format,
  document = true,
}) {
  if (document) {
    messages.unshift({
      role: "assistant",
      content: JSON.stringify({
        datasets: dataset.document(),
      }),
    });
  }
  messages.unshift({
    role: "assistant",
    content: JSON.stringify({ control_id: uuid() }),
  });

  return openrouter.generate({
    messages,
    response_format,
  });
}
