const { json } = require("../lib/Markdown");
const { GoogleGenerativeAI } = require("@google/generative-ai");

const gemini = new GoogleGenerativeAI(process.env.GEMINI_API_KEY);
const model = gemini.getGenerativeModel({
  model: "gemini-1.5-pro", // gemini-1.5-flash
  generationConfig: {
    temperature: 0,
  },
});

async function generate({ messages = [] }) {
  const result = await model.generateContent(
    messages.map((m) => m.content).join("\n")
  );

  return json(result.response.text());
}

module.exports = { generate };
