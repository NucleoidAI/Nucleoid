const express = require("express");
const cors = require("cors");
const fs = require("fs");
const context = require("./lib/context");
const openapi = require("./lib/openapi");

let nucleoid;

setImmediate(() => {
  nucleoid = require("../");
});

function app(options) {
  const app = express();
  app.use(express.json());
  app.use(cors());
  app.static = express.static;

  let listener;

  return {
    express: () => app,
    address: () => (listener ? listener.address() : null),
    use: (...args) => app.use(...args),
    get: (string, fn) => app.get(string, (req, res) => accept(req, res, fn)),
    post: (string, fn) => app.post(string, (req, res) => accept(req, res, fn)),
    put: (string, fn) => app.put(string, (req, res) => accept(req, res, fn)),
    delete: (string, fn) =>
      app.delete(string, (req, res) => accept(req, res, fn)),
    listen: (port = 3000, fn) => {
      app.all("*", (req, res) => res.status(404).end());
      // eslint-disable-next-line no-unused-vars
      app.use((err, req, res, next) => res.status(500).send(err.stack));

      nucleoid.start(options);
      return (listener = app.listen(port, fn));
    },
    context: (path) => {
      const file = fs.readFileSync(path, "utf8");
      const arr = JSON.parse(file);
      context.load(
        arr.map((item) => ({ ...item, options: { declarative: true } }))
      );
    },
    openapi: (path) => {
      try {
        const file = fs.readFileSync(path, "utf8");
        openapi.init(app);
        openapi.load(JSON.parse(file));
      } catch (err) {
        throw Error("Problem occurred while opening OpenAPI");
      }
    },
  };
}

function accept(req, res, fn) {
  const scope = { params: req.params, query: req.query, body: req.body };
  const { result, error } = nucleoid.run(fn, scope, { details: true });
  if (!result) res.status(404).end();
  else if (error) res.status(400).json(result);
  else res.status(200).json(result);
}

module.exports = app;
