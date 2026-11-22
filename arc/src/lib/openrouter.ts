import { HTTPClient, OpenRouter } from "@openrouter/sdk";
import {
  HTTPClientError,
  OpenRouterError,
  ResponseValidationError,
  SDKValidationError,
} from "@openrouter/sdk/models/errors";
import * as milestone from "./milestone.ts";

export async function generate({ reasoning = false, messages = [] }) {
  const apiKey: string = process.env.OPENROUTER_API_KEY?.trim();
  if (!apiKey) {
    throw new Error("OPENROUTER_API_KEY is required");
  }

  let cacheStatus = "";
  const httpClient = new HTTPClient().addHook("response", (response) => {
    cacheStatus = response.headers.get("x-openrouter-cache-status") ?? "";
  });

  // The SDK retries 5XX responses and connection errors for up to an hour by
  // default, so retries are disabled to keep each generation one request.
  const openrouter = new OpenRouter({
    apiKey,
    httpClient,
    retryConfig: { strategy: "none" },
  });

  const data = await openrouter.chat
    .send(
      {
        chatRequest: {
          model: "qwen/qwen3.8-27b",
          messages,
          temperature: 0,
          topP: 1,
          topK: 1,
          seed: 42,
          frequencyPenalty: 0,
          presencePenalty: 0,
          reasoning: { enabled: reasoning },
          responseFormat: { type: "json_object" },
          provider: {
            only: ["deepinfra/fp8"],
            allowFallbacks: false,
          },
        },
      },
      { headers: { "X-OpenRouter-Cache": "false" } }
    )
    .catch((error) => {
      // SDK errors hold the raw response, the request payload or the transport
      // error, so only their messages are rethrown, without the cause.
      if (
        error instanceof OpenRouterError ||
        error instanceof HTTPClientError ||
        error instanceof SDKValidationError
      ) {
        throw new Error(describe(error));
      }
      throw error;
    });

  if (cacheStatus.toUpperCase() === "HIT") {
    throw new Error("OpenRouter returned a cached response");
  }

  const message = data.choices[0]?.message;

  if (reasoning) {
    milestone.reasoning({ reasoning: message?.reasoning });
  }

  const content = message?.content;
  if (typeof content !== "string" || !content.trim()) {
    throw new Error("OpenRouter returned no text content");
  }

  const cachedTokens: number = data.usage?.promptTokensDetails?.cachedTokens;

  if (data.usage) {
    const { promptTokens, completionTokens } = data.usage;
    milestone.usage({
      prompt_tokens: promptTokens,
      completion_tokens: completionTokens,
      cached_tokens: cachedTokens,
    });
  }

  if (cachedTokens !== 0) {
    milestone.prompt_cache_alert({ cached_tokens: cachedTokens });
  }

  let jsonContent;

  try {
    // console.debug(JSON.stringify(JSON.parse(content)));
    jsonContent = JSON.parse(content);
  } catch (error) {
    throw new Error(
      `Failed to parse OpenRouter response as JSON: ${error.message} \n\nResponse content: ${content}`
    );
  }

  return jsonContent;
}

function describe(
  error: OpenRouterError | HTTPClientError | SDKValidationError
): string {
  if (error instanceof ResponseValidationError) {
    // OpenRouter reports an error raised during generation with status 200.
    const body = error.rawValue as { error?: { message?: unknown } } | null;
    return typeof body?.error?.message === "string"
      ? `OpenRouter error: ${body.error.message}`
      : `OpenRouter returned an invalid response: ${error.pretty()}\n\nResponse body: ${
          error.body
        }`;
  }
  if (error instanceof OpenRouterError) {
    return `OpenRouter request failed (${error.statusCode}): ${error.message}`;
  }
  // fetch() keeps the network reason, such as ENOTFOUND, in its own cause.
  const reason: unknown =
    error.cause instanceof Error ? error.cause.cause : undefined;
  return `OpenRouter request failed: ${error.message}${
    reason instanceof Error ? `: ${reason.message}` : ""
  }`;
}
