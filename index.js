const runtime = require("./src/runtime");
const Token = require("./src/utils/token");
const express = require("express");
const cors = require("cors");

const preset = [];

const terminal = express();
terminal.use(express.text({ type: "*/*" }));
terminal.use(cors());

terminal.post("/", (req, res) => {
  const details = runtime.process(req.body, { details: true });
  res.send(details);
});
terminal.all("*", (req, res) => res.status(404).end());
// eslint-disable-next-line no-unused-vars
terminal.use((err, req, res, next) => res.status(500).send(err.stack));

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

const run = (statement, p2, p3) => {
  if (typeof statement === "string") {
    const options = p2;
    return runtime.process(`${statement}`, options);
  } else {
    let scope = p2;
    let options = p3;
    const { args, fn } = parseFn(statement.toString());
    scope =
      scope && args.length ? `let ${args[0]}=${JSON.stringify(scope)};` : "";
    return runtime.process(`${scope}${fn}`, options);
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
  const { result } = run(fn, scope, { details: true });
  if (result === undefined) res.status(404).end();
  else res.send(Number.isInteger(result) ? result.toString() : result);
};

module.exports = () => ({
  express: () => app,
  use: (fn) => app.use(fn),
  get: (string, fn) => app.get(string, (req, res) => accept(req, res, fn)),
  post: (string, fn) => app.post(string, (req, res) => accept(req, res, fn)),
  put: (string, fn) => app.put(string, (req, res) => accept(req, res, fn)),
  delete: (string, fn) =>
    app.delete(string, (req, res) => accept(req, res, fn)),
  listen: (port, fn) => {
    app.all("*", (req, res) => res.status(404).end());
    // eslint-disable-next-line no-unused-vars
    app.use((err, req, res, next) => res.status(500).send(err.stack));

    start();
    app.listen(port, fn);
  },
});

module.exports.start = start;
module.exports.register = register;
module.exports.run = run;
