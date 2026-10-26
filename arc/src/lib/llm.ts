import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as openrouter from "./openrouter";
import type { SchemaObject } from "ajv";

export async function generate<
  Response extends {
    declarations?: string[];
    instances?: {
      input_instance_matrix: number[][];
      output_instance_matrix?: number[][];
    }[];
    implementation?: string;
    orientation?: number;
    output_instance_matrix?: number[][];
  }
>({
  messages = [],
  response_format,
  document = true,
}: {
  messages?: {
    role: "system" | "user" | "assistant";
    content: string;
  }[];
  response_format: {
    type: "json_schema";
    json_schema: {
      name: string;
      strict: true;
      schema: {
        type: "object";
        properties: { [property: string]: SchemaObject };
        required: string[];
        additionalProperties: false;
      };
    };
  };
  document?: boolean;
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

  return openrouter.generate<Response>({
    messages,
    response_format,
  });
}
