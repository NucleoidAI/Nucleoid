const express = require("express");
const router = express.Router();
const openapi = require("../lib/openapi");
const context = require("../lib/context");
const config = require("../config");

router.get("/openapi", (req, res) => res.json(openapi.status()));
router.post("/openapi", (req, res) => {
  // TODO Change contract to OpenAPI
  const { api, types, functions, action, port, prefix } = req.body;
  const _config = config();

  if (action === "start") {
    context.run(functions);

    openapi.init();
    openapi.load({ api, types, prefix });
    openapi.start(port || _config.port.openapi);
  } else if (action === "stop") {
    openapi.stop();
  }

  res.end();
});

module.exports = router;
