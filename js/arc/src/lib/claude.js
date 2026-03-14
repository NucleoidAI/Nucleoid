const Anthropic = require("@anthropic-ai/sdk");

const anthropic = new Anthropic({
  apiKey: process.env.CLAUDE_API_KEY,
});

async function generate({
  model = "claude-3-5-sonnet-20240620", // claude-3-haiku-20240307
  messages = [],
  temperature = 0,
  max_tokens = 2048,
}) {
  const {
    content: [{ text }],
    usage,
  } = await anthropic.messages.create({
    model,
    messages,
    temperature,
    max_tokens,
  });

  console.info(usage);
  return JSON.parse(text);
}

module.exports = { generate };
