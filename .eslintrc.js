module.exports = {
  env: {
    es6: true,
    node: true,
    mocha: true,
  },
  extends: ["eslint:recommended", "prettier"],
  parserOptions: {
    ecmaVersion: 2018,
    sourceType: "module",
  },
  ignorePatterns: "/node_modules",
  rules: {
    eqeqeq: ["error", "always"],
    "no-console": "off",
    "no-eval": "error",
    "no-var": "error",
  },
};
