const escodegen = require("escodegen");
const { FORMAT_MINIFY } = require("escodegen");

function generate(node) {
  return escodegen.generate(node, { format: FORMAT_MINIFY });
}

module.exports.generate = generate;
