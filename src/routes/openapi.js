const express = require("express");
const router = express.Router();
const openapi = require("../lib/openapi");
const context = require("../lib/context");

router.get("/openapi", (req, res) => res.json(openapi.status()));
router.post("/openapi", (req, res) => {
  const { api, types, functions, action, port, prefix } = req.body;

  if (action === "start") {
    context.run(functions);

    openapi.init();
    openapi.load({ api, types, prefix });
    openapi.start(port);
  } else if (action === "stop") {
    openapi.stop();
  }

  res.end();
});

module.exports = router;
