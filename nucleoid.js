var stack = require("./stack");
var Statement = require("./statement");
const fs = require("fs");
const argv = require("yargs").argv;

module.exports.run = function(string, details, cacheOnly) {
  let before = Date.now();
  let result, statements;

  try {
    statements = Statement.compile(string);
    result = stack.process(statements);
  } catch (error) {
    if (error instanceof Error) {
      result = error.message;
    } else {
      result = error;
    }

    if (!details) {
      throw error;
    }
  }

  if (argv.id !== undefined && !cacheOnly) {
    fs.appendFileSync(
      "./data/" + argv.id,
      JSON.stringify({
        s: string,
        t: Date.now() - before,
        r: result instanceof Object ? JSON.stringify(result) : result,
        d: Date.now()
      }) + "\n"
    );
  }

  if (details) {
    return { result, statements, time: Date.now() - before };
  } else {
    return result;
  }
};
