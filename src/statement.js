const JS = require("./lang/js/js");

function compile(string) {
  return JS.compile(string).statements;
}

module.exports.compile = compile;
