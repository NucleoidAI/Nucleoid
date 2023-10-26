const acorn = require("./lang/estree/acorn");

function parse(string) {
  return acorn.parse(string);
}

module.exports.parse = parse;
