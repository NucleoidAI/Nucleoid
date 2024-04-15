const express = require("express");
const cors = require("cors");
const graph = require("./graph");
const openapi = require("./openapi");
const logs = require("./logs");
const metrics = require("./metrics");
const runtime = require("../runtime");

const terminal = express();
terminal.use(express.json());
terminal.use(express.text({ type: "application/javascript" }));
terminal.use(cors());

terminal.use(graph);
terminal.use(openapi);
terminal.use(logs);
terminal.use(metrics);

terminal.post("/", (req, res) => {
  const mode = req.headers["x-nuc-mode"];
  const declarative = mode?.toLowerCase() === "declarative";
  const details = runtime.process(req.body, { declarative, details: true });

  res.send(details);
});

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
