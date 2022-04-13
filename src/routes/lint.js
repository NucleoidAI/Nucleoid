const express = require("express");
const router = express.Router();

const { ESLint } = require("eslint");
const eslint = new ESLint({
  baseConfig: {
    extends: ["eslint:recommended", "plugin:prettier/recommended"],
  },
  overrideConfig: {
    rules: {
      "no-unused-vars": [
        "error",
        {
          varsIgnorePattern: "action",
          args: "none",
        },
      ],
    },
  },
  fix: true,
});

router.post("/lint", async (req, res) => {
  const [result] = await eslint.lintText(req.body);
  res.json(result);
});

module.exports = router;
