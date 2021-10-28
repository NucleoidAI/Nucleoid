const express = require("express");
const bodyParser = require("body-parser");
const cors = require("cors");
const Service = require("./service");

const app = express();
app.use(bodyParser.text({ type: "*/*" }));
app.use(cors());

Service.start("main");

app.post("/", (req, res) => Service.accept(req.body, req, res));
app.use(function (err, res) {
  res.type("txt");
  res.status(500).send(err.stack);
});

app.listen(8448);
