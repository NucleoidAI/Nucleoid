const axios = require("axios").default;

async function generate({ messages = [] }) {
  const { data } = await axios({
    method: "POST",
    url: `http://${process.env.LLAMA_HOST}/generate`,
    headers: {
      "Content-Type": "application/json",
    },
    data: {
      input: messages.map((m) => m.content).join("\n"),
    },
  });

  return data;
}

module.exports = { generate };
