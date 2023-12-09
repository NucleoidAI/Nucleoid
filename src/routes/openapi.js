const express = require("express");
const router = express.Router();
const _openapi = require("../lib/openapi");
const context = require("../lib/context");
const config = require("../config");

router.get("/openapi", (req, res) => res.json(_openapi.status()));
router.post("/openapi", (req, res) => {
  const { openapi, action, port, prefix, events = [] } = req.body;
  const _config = config();

  if (action === "start") {
    context.run(openapi.functions);
    delete openapi.functions;

    _openapi.init();
    _openapi.load({ openapi, prefix, events });
    _openapi.start(port || _config.port.openapi);
  } else if (action === "stop") {
    _openapi.stop();
  }

  res.end();
});

module.exports = router;
