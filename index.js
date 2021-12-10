const runtime = require("./src/runtime");
const Token = require("./src/utils/token");
const express = require("express");
const bodyParser = require("body-parser");
const cors = require("cors");

const preset = [];

const start = (options) => {
  options = options || {};

  const process = require("./src/process");
  process.options(options);

  const { terminal } = options;

  setImmediate(() => {
    preset.forEach(({ fn, options }) => run(fn.toString(), options));
  });

  if (terminal === undefined || terminal === true) {
    app.listen(8448);
  }
};

const register = (fn, options) => {
  preset.push({ fn, options });
};

const run = (statement, p2, p3) => {
  if (typeof statement === "string") {
    const options = p2;
    return runtime.process(statement, options);
  } else {
    const scope = p2 || {};
    const options = p3;

    const string = statement.toString();
    let context = Token.next(string, 0);
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
    context = Token.nextBlock(string, context.offset);

    const exec =
      Object.entries(scope)
        .map(([key, value]) => `let ${key}=${JSON.stringify(value)};`)
        .join("") + context.block.trim();

    return runtime.process(exec, options);
  }
};

const app = express();
app.use(bodyParser.text({ type: "*/*" }));
app.use(cors());

app.post("/", (req, res) => res.send(runtime.process(req.body)));
app.use((err, res) => {
  res.type("txt");
  res.status(500).send(err.stack);
});

module.exports = { start, register, run };
