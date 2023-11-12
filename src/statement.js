const ESTree = require("./lang/estree/parser");

function parse(string) {
  return ESTree.parse(string);
}

module.exports.parse = parse;
