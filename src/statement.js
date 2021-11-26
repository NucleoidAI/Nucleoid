const ES6 = require("./lang/es6/es6");

module.exports.compile = function (string) {
  return ES6.compile(string).statements;
};
