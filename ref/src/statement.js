const ESTree = require("./lang/estree/parser");

function compile(string) {
  return ESTree.parse(string);
}

module.exports.compile = compile;
