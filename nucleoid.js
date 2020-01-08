var stack = require("./stack");
var Statement = require("./statement");
const fs = require("fs");

module.exports.run = function(string, details, cacheOnly) {
  let before = Date.now();

  let statements = Statement.compile(string);
  let result = stack.process(statements);

  if (!cacheOnly) {
    fs.appendFileSync("data", string + "\n");
  }

  let after = Date.now();

  if (details === true) {
    return { result, statements, time: after - before };
  } else {
    return result;
  }
};
