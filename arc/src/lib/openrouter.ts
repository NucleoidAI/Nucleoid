import axios from "axios";
import { isObject } from "./json";
import * as terminal from "./terminal";

function optionalString(value: unknown): value is string | null | undefined {
  return value === undefined || value === null || typeof value === "string";
}

function optionalError(
  value: unknown
): value is { message: string } | null | undefined {
  return (
    value === undefined ||
    value === null ||
    (isObject(value) && typeof value.message === "string")
  );
}

function validateEnvelope(value: unknown): value is {
  id?: string | null;
  model?: string | null;
  provider?: string | null;
  error?: { message: string } | null;
  choices?:
    | {
        finish_reason?: string | null;
        native_finish_reason?: string | null;
        error?: { message: string } | null;
        message?: { content?: unknown; refusal?: unknown } | null;
      }[]
    | null;
  usage?: { prompt_tokens: number; completion_tokens: number } | null;
} {
  return (
    isObject(value) &&
    optionalString(value.id) &&
    optionalString(value.model) &&
    optionalString(value.provider) &&
    optionalError(value.error) &&
    (value.choices === undefined ||
      value.choices === null ||
      (Array.isArray(value.choices) &&
        value.choices.every(
          (choice: unknown) =>
            isObject(choice) &&
            optionalString(choice.finish_reason) &&
            optionalString(choice.native_finish_reason) &&
            optionalError(choice.error) &&
            (choice.message === undefined ||
              choice.message === null ||
              isObject(choice.message))
        ))) &&
    (value.usage === undefined ||
      value.usage === null ||
      (isObject(value.usage) &&
        typeof value.usage.prompt_tokens === "number" &&
        Number.isFinite(value.usage.prompt_tokens) &&
        typeof value.usage.completion_tokens === "number" &&
        Number.isFinite(value.usage.completion_tokens)))
  );
}

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
        properties: { [property: string]: { [keyword: string]: unknown } };
        required: string[];
        additionalProperties: false;
      };
    };
  };
}) {
  const name: string = response_format.json_schema.name;
  const schema = response_format.json_schema.schema;
  const apiKey: string = process.env.OPENROUTER_API_KEY?.trim();
  if (!apiKey) {
    throw new Error("OPENROUTER_API_KEY is required");
  }

  const { data } = await axios<unknown>({
    method: "POST",
    url: "https://openrouter.ai/api/v1/chat/completions",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${apiKey}`,
    },
    data: {
      model: "qwen/qwen3.8-27b",
      messages: [
        {
          role: "system",
          content: JSON.stringify({ json_format: schema }),
        },
        ...messages,
      ],
      temperature: 0,
      top_p: 1,
      top_k: 1,
      seed: 42,
      frequency_penalty: 0,
      presence_penalty: 0,
      reasoning: { enabled: false },
      response_format,
      provider: {
        only: ["deepinfra/bf16"],
        allow_fallbacks: false,
        require_parameters: true,
      },
    },
  }).catch((error: unknown) => {
    if (!axios.isAxiosError<unknown>(error)) {
      throw error;
    }
    const status: string = error.response ? ` (${error.response.status})` : "";
    const body = error.response?.data;
    const message = validateEnvelope(body) ? body.error?.message : undefined;
    // Axios errors contain authentication headers, so do not attach the cause.
    throw new Error(
      `OpenRouter request failed${status}: ${message ?? error.message}`
    );
  });

  if (!validateEnvelope(data)) {
    throw new Error(
      `OpenRouter returned an invalid completion envelope for ${name}`
    );
  }

  if (data.error) {
    throw new Error(`OpenRouter error: ${data.error.message}`);
  }

  const choice = data.choices?.[0];
  const completion: string = JSON.stringify({
    schema: name,
    id: data.id ?? null,
    model: data.model ?? null,
    provider: data.provider ?? null,
    finish_reason: choice?.finish_reason ?? null,
    native_finish_reason: choice?.native_finish_reason ?? null,
    completion_tokens: data.usage?.completion_tokens ?? null,
  });
  if (data.usage) {
    const { prompt_tokens, completion_tokens } = data.usage;
    terminal.usage({ prompt_tokens, completion_tokens });
  }

  if (choice?.error) {
    throw new Error(
      `OpenRouter completion failed: ${choice.error.message}\n\nCompletion: ${completion}`
    );
  }
  if (choice?.message?.refusal) {
    throw new Error(
      `OpenRouter refused the response: ${JSON.stringify(
        choice.message.refusal
      )}\n\nCompletion: ${completion}`
    );
  }
  if (choice?.finish_reason !== "stop") {
    const reason: string =
      choice?.finish_reason === "length"
        ? "response was truncated"
        : "completion did not finish normally";
    throw new Error(`OpenRouter ${reason}\n\nCompletion: ${completion}`);
  }

  const content = choice.message?.content;
  if (typeof content !== "string" || !content.trim()) {
    throw new Error(
      `OpenRouter returned no text content\n\nCompletion: ${completion}`
    );
  }

  let jsonContent: Response;

  try {
    jsonContent = JSON.parse(content);
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(
      `Failed to parse OpenRouter response as JSON: ${message}\n\nCompletion: ${completion}\n\nResponse content: ${content}`
    );
  }

  return jsonContent;
}
