const express = require("express");
const router = express.Router();
const openapi = require("../libs/openapi");

router.get("/openapi", (req, res) => res.json(openapi.status()));
router.post("/openapi", (req, res) => {
  const { nuc, action } = req.body;

  if (action === "start") {
    openapi.start(nuc);
  } else if (action === "stop") {
    openapi.stop();
  }

  res.end();
});

module.exports = router;
