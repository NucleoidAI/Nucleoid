const axios = require("axios").default;

async function run(sessionId, data) {
  const response = await axios({
    method: "POST",
    url: `https://nuc.land/sandbox/terminal/${sessionId}`,
    headers: {
      "Content-Type": "application/javascript",
    },
    data,
  });

  return response.data.result;
}

module.exports = { run };
