const express = require("express");
const router = express.Router();
const _openapi = require("../lib/openapi");
const context = require("../context");
const config = require("../config");

router.get("/openapi", (req, res) => res.json(_openapi.status()));
router.post("/openapi", (req, res) => {
  const openapi = req.body;
  const _config = config();

  if (openapi["x-nuc-action"] === "start") {
    context.run(openapi["x-nuc-functions"]);

    _openapi.init();
    _openapi.load(openapi);
    _openapi.start(openapi["x-nuc-port"] || _config.port.openapi);
  } else if (openapi["x-nuc-action"] === "stop") {
    _openapi.stop();
  }

  res.end();
});

module.exports = router;
