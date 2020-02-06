var stack = require("./stack");
var Statement = require("./statement");
const fs = require("fs");
const argv = require("yargs").argv;

module.exports.run = function(string, details) {
  let before = Date.now();

  let statements = Statement.compile(string);
  let result = stack.process(statements);

  if (argv.id !== undefined) {
    fs.appendFileSync("./data/" + argv.id, string + "\n");
  }

  let after = Date.now();

  if (details === true) {
    return { result, statements, time: after - before };
  } else {
    return result;
  }
};
