const runtime = require("./src/runtime");
const Token = require("./src/utils/token");
const express = require("express");
const cors = require("cors");

const preset = [];

const terminal = express();
terminal.use(express.text({ type: "*/*" }));
terminal.use(cors());

terminal.post("/", (req, res) => res.send(runtime.process(req.body)));
terminal.use((err, res) => {
  res.type("txt");
  res.status(500).send(err.stack);
});

const start = (options) => {
  options = options || {};

  const process = require("./src/process");
  options = process.options(options);

  setImmediate(() => {
    preset.forEach(({ fn, options }) => run(fn.toString(), options));
  });

  if (options.terminal === undefined || options.terminal === true) {
    terminal.listen(8448);
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
    const { args, fn } = parseFn(statement.toString());
    scope =
      scope && args.length ? `let ${args[0]}=${JSON.stringify(scope)};` : "";
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
app.use(express.json());
app.use(cors());

const accept = (req, res, fn) => {
  const scope = { params: req.params, query: req.query, body: req.body };
  const result = run(fn, scope);
  if (result === undefined) res.status(404).end();
  else res.send(result);
};

module.exports = () => ({
  use: (fn) => app.use(fn),
  get: (string, fn) => app.get(string, (req, res) => accept(req, res, fn)),
  post: (string, fn) => app.post(string, (req, res) => accept(req, res, fn)),
  put: (string, fn) => app.put(string, (req, res) => accept(req, res, fn)),
  delete: (string, fn) =>
    app.delete(string, (req, res) => accept(req, res, fn)),
  listen: (port, fn) => {
    app.all("*", (req, res) => {
      res.status(404).end();
    });

    app.use((err, req, res, next) => {
      if (!err) next();
      else if (!(err instanceof Error)) res.status(400).send({ error: err });
      else res.status(500).send({ error: err.message });
    });

    start();
    app.listen(port, fn);
  },
});

module.exports.start = start;
module.exports.register = register;
module.exports.run = run;
