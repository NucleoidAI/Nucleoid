const runtime = require("./src/runtime");
const express = require("express");
const cors = require("cors");
const openapiRoute = require("./src/routes/openapi");
const logs = require("./src/routes/logs");
const metrics = require("./src/routes/metrics");
const lint = require("./src/routes/lint");
const parser = require("./src/libs/parser");
const openapi = require("./src/libs/openapi");
const fs = require("fs");
const context = require("./src/libs/context");

const start = (options = {}) => {
  const process = require("./src/process");
  options = process.options(options);
  setImmediate(() => context.run());

  if (options.terminal !== false && options.test !== false) {
    const terminal = express();
    terminal.use(express.json());
    terminal.use(express.text({ type: "*/*" }));
    terminal.use(cors());

    terminal.use(openapiRoute);
    terminal.use(logs);
    terminal.use(metrics);
    terminal.use(lint);

    terminal.post("/", (req, res) => {
      const details = runtime.process(req.body, { details: true });
      res.send(details);
    });
    terminal.all("*", (req, res) => res.status(404).end());

    // eslint-disable-next-line no-unused-vars
    terminal.use((err, req, res, next) => res.status(500).send(err.stack));
    terminal.listen(8448);
  }
};

const register = (fn, options) => {
  context.load({ definition: fn.toString(), options });
};

const run = (statement, p2, p3) => {
  if (typeof statement === "string") {
    const options = p2;
    return runtime.process(`${statement}`, options);
  } else {
    let scope = p2;
    let options = p3;
    const { args, fn } = parser.fn(statement.toString());
    scope =
      scope && args.length ? `let ${args[0]}=${JSON.stringify(scope)};` : "";
    return runtime.process(`${scope}${fn}`, options);
  }
};

const app = express();
app.use(express.json());
app.use(cors());
app.static = express.static;

let listener;

const accept = (req, res, fn) => {
  const scope = { params: req.params, query: req.query, body: req.body };
  const { result } = run(fn, scope, { details: true });
  if (!result) res.status(404).end();
  else if (result === true) res.end();
  else res.send(Number.isInteger(result) ? result.toString() : result);
};

module.exports = (options) => ({
  express: () => app,
  address: () => (listener ? listener.address() : null),
  use: (...args) => app.use(...args),
  get: (string, fn) => app.get(string, (req, res) => accept(req, res, fn)),
  post: (string, fn) => app.post(string, (req, res) => accept(req, res, fn)),
  put: (string, fn) => app.put(string, (req, res) => accept(req, res, fn)),
  delete: (string, fn) =>
    app.delete(string, (req, res) => accept(req, res, fn)),
  listen: (port = 0, fn) => {
    app.all("*", (req, res) => res.status(404).end());
    // eslint-disable-next-line no-unused-vars
    app.use((err, req, res, next) => res.status(500).send(err.stack));

    start(options);
    return (listener = app.listen(port, fn));
  },
  context: (path) => {
    const file = fs.readFileSync(path, "utf8");
    context.load(JSON.parse(file));
  },
  openapi: (path) => {
    try {
      const file = fs.readFileSync(path, "utf8");
      openapi.initialize(app);
      openapi.load(JSON.parse(file));
    } catch (err) {
      throw Error("Problem occurred while opening OpenAPI");
    }
  },
});

module.exports.start = start;
module.exports.register = register;
module.exports.run = run;
