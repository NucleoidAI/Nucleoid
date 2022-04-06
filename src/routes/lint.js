const express = require("express");
const router = express.Router();
const Linter = require("eslint").Linter;
const linter = new Linter();

const options = require("./lintOptions");

router.post("/lint", (req, res) => {
  res.json(linter.verifyAndFix(req.body, options));
});

module.exports = router;
