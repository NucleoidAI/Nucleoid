module.exports = {
  env: {
    es6: true,
    node: true,
    mocha: true
  },
  extends: "eslint:recommended",
  parserOptions: {
    ecmaVersion: 2018,
    sourceType: "module"
  },
  rules: {
    eqeqeq: ["error", "always"],
    "no-console": "off"
  }
};
