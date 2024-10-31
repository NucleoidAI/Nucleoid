const axios = require("axios").default;

async function generate({
  messages = [],
  temperature = 0,
  top_p = 1,
  presence_penalty = 0,
  frequency_penalty = 0,
}) {
  const {
    data: {
      choices: [
        {
          message: { content },
        },
      ],
      usage: { prompt_tokens, completion_tokens },
    },
  } = await axios({
    method: "POST",
    url: process.env.AZURE_ENDPOINT,
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${process.env.AZURE_API_KEY}`,
    },
    data: {
      messages,
      temperature,
      top_p,
      presence_penalty,
      frequency_penalty,
    },
  });

  console.info({ prompt_tokens, completion_tokens });
  return JSON.parse(content);
}

module.exports = { generate };
