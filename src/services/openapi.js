const express = require("express");
const router = express.Router();
const openapi = require("../libs/openapi");
const context = require("../libs/context");

router.get("/openapi", (req, res) => res.json(openapi.status()));
router.post("/openapi", (req, res) => {
  const { api, types, functions, action, port } = req.body;

  if (action === "start") {
    context.run(functions);

    openapi.initialize();
    openapi.load({ api, types });
    openapi.start(port);
  } else if (action === "stop") {
    openapi.stop();
  }

  res.end();
});

module.exports = router;
