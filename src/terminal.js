const express = require("express");
const cors = require("cors");
const graph = require("./routes/graph");
const openapi = require("./routes/openapi");
const logs = require("./routes/logs");
const metrics = require("./routes/metrics");
const runtime = require("./runtime");

const terminal = express();
terminal.use(express.json());
terminal.use(express.text({ type: "*/*" }));
terminal.use(cors());

terminal.use(graph);
terminal.use(openapi);
terminal.use(logs);
terminal.use(metrics);

terminal.post("/", (req, res) => {
  const details = runtime.process(req.body, { details: true });
  res.send(details);
});

// eslint-disable-next-line no-unused-vars
terminal.use((err, req, res, next) => {
  if (typeof err === "string") {
    res.status(400).json({ error: err });
  } else {
    res.status(500).send(err.stack);
  }
});

module.exports = terminal;
