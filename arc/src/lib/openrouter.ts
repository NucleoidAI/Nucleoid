import axios from "axios";
import Ajv from "ajv";
import * as terminal from "./terminal";

const ajv = new Ajv({ allErrors: true });

function requireCode(source: string, path: string, completion: string) {
  if (!source.trim()) {
    throw new Error(
      `OpenRouter returned blank Nucleoid code at ${path}\n\nCompletion: ${completion}`
    );
  }
}

export async function generate({ messages = [], response_format }) {
  const name: string = response_format.json_schema.name;
  const schema = response_format.json_schema.schema;
  const validate = ajv.compile(schema);
  const apiKey: string = process.env.OPENROUTER_API_KEY?.trim();
  if (!apiKey) {
    throw new Error("OPENROUTER_API_KEY is required");
  }

  const { data } = await axios({
    method: "POST",
    url: "https://openrouter.ai/api/v1/chat/completions",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${apiKey}`,
    },
    data: {
      model: "qwen/qwen3.8-27b",
      messages,
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
  }).catch((error) => {
    if (!axios.isAxiosError(error)) {
      throw error;
    }
    const status: string = error.response ? ` (${error.response.status})` : "";
    const message = error.response?.data?.error?.message;
    // Axios errors contain authentication headers, so do not attach the cause.
    throw new Error(
      `OpenRouter request failed${status}: ${
        typeof message === "string" ? message : error.message
      }`
    );
  });

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

  const content: string = choice.message?.content;
  if (typeof content !== "string" || !content.trim()) {
    throw new Error(
      `OpenRouter returned no text content\n\nCompletion: ${completion}`
    );
  }

  let jsonContent;

  try {
    jsonContent = JSON.parse(content);
  } catch (error) {
    throw new Error(
      `Failed to parse OpenRouter response as JSON: ${error.message}\n\nCompletion: ${completion}\n\nResponse content: ${content}`
    );
  }

  const valid: boolean = validate(jsonContent);
  if (!valid) {
    throw new Error(
      `OpenRouter response does not match ${name}: ${JSON.stringify(
        validate.errors
      )}\n\nCompletion: ${completion}`
    );
  }

  jsonContent.declarations?.forEach((source: string, index: number) =>
    requireCode(source, `/declarations/${index}`, completion)
  );
  if (Object.hasOwn(jsonContent, "implementation")) {
    requireCode(jsonContent.implementation, "/implementation", completion);
  }

  return jsonContent;
}
