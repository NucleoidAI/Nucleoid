const express = require("express");
const router = express.Router();
const os = require("os");

router.get("/metrics", (req, res) =>
  res.json({
    free: os.freemem(),
    total: os.totalmem(),
  })
);

module.exports = router;
