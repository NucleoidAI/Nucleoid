const runtime = require("./src/runtime");
const Token = require("./src/utils/token");
const express = require("express");
const cors = require("cors");

const preset = [];

const start = (options) => {
  options = options || {};

  const process = require("./src/process");
  const { terminal } = process.options(options);

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

const run = (statement, p2) => {
  if (typeof statement === "string") {
    const options = p2;
    return runtime.process(`${statement}`, options);
  } else {
    let scope = p2;
    scope = scope ? `let scope=${JSON.stringify(scope)};` : "";

    const { fn } = parseFn(statement.toString());
    return runtime.process(`${scope}${fn}`);
  }
};

const parseFn = (string) => {
  let args;
  let fn;

  let context = Token.next(string, 0);

  if (context.token === "(") {
    context = Token.nextArgs(string, context.offset);
    args = context.args;
  } else {
    args = [context.token];
  }

  context = Token.next(string, context.offset);
  context = Token.next(string, context.offset);

  let check = Token.next(string, context.offset);

  if (check && check.token === "{") {
    context = Token.nextBlock(string, check.offset);
    fn = `{${context.block.trim()}}`;
  } else {
    context = Token.nextBlock(string, context.offset, true);
    fn = context.block.trim();
  }

  return { args, fn };
};

const app = express();
app.use(express.text({ type: "*/*" }));
app.use(cors());

app.post("/", (req, res) => res.send(runtime.process(req.body)));
app.use((err, res) => {
  res.type("txt");
  res.status(500).send(err.stack);
});

module.exports = { start, register, run };
