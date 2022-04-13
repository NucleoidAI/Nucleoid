const express = require("express");

const router = express.Router();

const options = require("./lintOptions");

const { ESLint } = require("eslint");
const eslint = new ESLint({ baseConfig: options, fix: true });

router.post("/lint", async (req, res) => {
  res.send(await eslint.lintText(req.body));
});

module.exports = router;
