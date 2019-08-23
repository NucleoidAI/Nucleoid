var stack = require("./stack");
var Statement = require("./statement");
const fs = require("fs");

module.exports.run = function(string) {
  let statements = Statement.compile(string);
  fs.appendFileSync("data", string + "\n");
  return stack.process(statements);
};
