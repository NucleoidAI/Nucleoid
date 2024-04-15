const express = require("express");
const cors = require("cors");
const graph = require("./graph");
const openapi = require("./openapi");
const logs = require("./logs");
const metrics = require("./metrics");
const runtime = require("../runtime");

const terminal = express();
terminal.use(cors());

terminal.post(
  "/",
  (req, res, next) =>
    req.is("application/javascript") ? next() : res.status(415).end(),
  express.text({ type: "application/javascript" }),
  (req, res) => {
    const details = runtime.process(req.body, { details: true });
    res.send(details);
  }
);

terminal.use(express.json(), (err, req, res, next) =>
  err ? res.status(422).end() : next()
);
terminal.use(graph);
terminal.use(openapi);
terminal.use(logs);
terminal.use(metrics);

// eslint-disable-next-line no-unused-vars
terminal.use((err, req, res, next) => {
  if (typeof err === "string") {
    res.status(400).json({ error: err });
  } else if (
    err instanceof SyntaxError &&
    err.status === 400 &&
    "body" in err
  ) {
    res.status(400).json({ error: "INVALID_JSON" });
  } else {
    res.status(500).send(err.stack);
  }
});

module.exports = terminal;
