const express = require("express");
const router = express.Router();

const { ESLint } = require("eslint");
const eslint = new ESLint({
  baseConfig: {
    extends: ["eslint:recommended", "plugin:prettier/recommended", "prettier"],
    plugins: ["prettier"],
  },
  overrideConfig: { rules: { "no-unused-vars": "off" } },
  fix: true,
});

router.post("/lint", async (req, res) => {
  res.send(await eslint.lintText(req.body));
});

module.exports = router;
