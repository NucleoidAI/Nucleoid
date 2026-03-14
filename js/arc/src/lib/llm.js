const { v4: uuid } = require("uuid");
const instruct_dataset = require("../instruct_dataset");

let llm;

if (process.env.LLM === "CLAUDE") {
  llm = require("./claude");
}

if (process.env.LLM === "OPENAI") {
  llm = require("./openai");
}

if (process.env.LLM === "GEMINI") {
  llm = require("./gemini");
}

if (process.env.LLM === "MISTRAL") {
  llm = require("./mistral");
}

if (process.env.LLM === "LLAMA") {
  llm = require("./llama");
}

if (process.env.LLM === "AZURE") {
  llm = require("./azure");
}

async function generate({ model, messages = [], temperature = 0, max_tokens }) {
  messages.unshift({
    role: "user",
    content: instruct_dataset.document(),
  });
  messages.unshift({
    role: "user",
    content: `
      control_id:
      ${uuid()}
      instruction:
      Respond in given json_format without any comments
    `,
  });

  return llm.generate({
    model,
    messages,
    temperature,
    max_tokens,
  });
}

module.exports = { generate };
