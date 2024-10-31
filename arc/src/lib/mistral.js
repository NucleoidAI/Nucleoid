const { Mistral } = require("@mistralai/mistralai");
const { json } = require("../lib/Markdown");

const mistral = new Mistral({
  apiKey: process.env.MISTRAL_API_KEY,
});

async function generate({
  model = "mistral-large-latest", // mistral-small-latest
  messages = [],
  temperature = 0,
  max_tokens = 2048,
}) {
  const {
    choices: [
      {
        message: { content },
      },
    ],
    usage: { promptTokens: prompt_tokens, completionTokens: completion_tokens },
  } = await mistral.chat.complete({
    model,
    messages,
    temperature,
    maxTokens: max_tokens,
  });

  console.info({ prompt_tokens, completion_tokens });
  return json(content);
}

module.exports = { generate };
