const JS = require("./lang/js/js");

module.exports.compile = function (string) {
  return JS.compile(string).statements;
};
