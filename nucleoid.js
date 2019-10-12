var stack = require("./stack");
var Statement = require("./statement");

module.exports.run = function(string) {
  let statements = Statement.compile(string);
  return stack.process(statements);
};
