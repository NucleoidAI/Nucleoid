const express = require("express");
const cors = require("cors");
const service = require("./service");
const logs = require("./src/services/logs");
const metrics = require("./src/services/metrics");

const terminal = express();
terminal.use(express.json());
terminal.use(express.text({ type: "*/*" }));
terminal.use(cors());

service.start("main");

terminal.use(logs);
terminal.use(metrics);

terminal.post("/", (req, res) => service.accept(req.body, req, res));
terminal.all("*", (req, res) => res.status(404).end());
// eslint-disable-next-line no-unused-vars
terminal.use((err, req, res, next) => res.status(500).send(err.stack));

terminal.listen(8448);
