const express = require("express");
const cors = require("cors");
const service = require("./service");

const app = express();
app.use(express.text({ type: "*/*" }));
app.use(cors());

service.start("main");

app.post("/", (req, res) => service.accept(req.body, req, res));
app.use((err, res) => {
  res.type("txt");
  res.status(500).send(err.stack);
});

app.listen(8448);
