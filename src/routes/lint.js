const express = require("express");
const router = express.Router();

const { Linter } = require("eslint");
const linter = new Linter();

const rules = require("./rules");

const options = {
  env: {
    es6: true,
    node: true,
  },
  parserOptions: {
    ecmaVersion: 2018,
    sourceType: "module",
  },
  rules,
};

router.post("/lint", async (req, res) => {
  const result = linter.verifyAndFix(req.body, options);
  res.json(result);
});

module.exports = router;
