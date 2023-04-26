const JS = require("./lang/js/JS");

function compile(string) {
  return JS.compile(string).statements;
}

module.exports.compile = compile;
