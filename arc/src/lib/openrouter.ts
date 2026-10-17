import axios from "axios";
import type { GenerateOptions } from "../types";

interface OpenRouterResponse {
  choices?: { message?: { content?: string | null } }[];
  usage?: { prompt_tokens: number; completion_tokens: number };
  error?: { message: string };
}

export async function generate<T = unknown>({
  messages = [],
}: GenerateOptions): Promise<T> {
  const apiKey = process.env.OPENROUTER_API_KEY?.trim();
  if (!apiKey) {
    throw new Error("OPENROUTER_API_KEY is required");
  }

  const { data } = await axios<OpenRouterResponse>({
    method: "POST",
    url: "https://openrouter.ai/api/v1/chat/completions",
    headers: {
      "Content-Type": "application/json",
      "X-OpenRouter-Cache": "false",
      Authorization: `Bearer ${apiKey}`,
    },
    data: {
      model: "qwen/qwen3.8-27b",
      messages,
      temperature: 0,
      max_tokens: 2048,
      top_p: 1,
      top_k: 1,
      seed: 42,
      frequency_penalty: 0,
      presence_penalty: 0,
      reasoning: { enabled: false },
      response_format: { type: "json_object" },
    },
  }).catch((error: unknown) => {
    if (!axios.isAxiosError<OpenRouterResponse>(error)) {
      throw error;
    }
    const status = error.response ? ` (${error.response.status})` : "";
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
  const content = data.choices?.[0]?.message?.content;
  if (typeof content !== "string" || !content.trim()) {
    throw new Error("OpenRouter returned no text content");
  }

  if (data.usage) {
    const { prompt_tokens, completion_tokens } = data.usage;
    console.info({ prompt_tokens, completion_tokens });
  }
  return JSON.parse(content);
}
