const express = require("express");
const cors = require("cors");
const service = require("./service");

const terminal = express();
terminal.use(express.text({ type: "*/*" }));
terminal.use(cors());

service.start("main");

terminal.post("/", (req, res) => service.accept(req.body, req, res));
terminal.use((err, res) => {
  res.type("txt");
  res.status(500).send(err.stack);
});

terminal.listen(8448);
