var Stack = require("./stack");
var Statement = require("./statement");
const fs = require("fs");
const argv = require("yargs").argv;

module.exports.run = function(string, details, cacheOnly) {
  let before = Date.now();
  let error, message;

  try {
    var statements = Statement.compile(string);
    var result = Stack.process(statements);
  } catch (e) {
    result = e;
    error = true;
  }

  if (argv.id !== undefined && !cacheOnly) {
    try {
      if (result instanceof Error) {
        message = JSON.stringify(result.message);
      } else {
        message = JSON.stringify(result);
      }
    } catch (e) {
      message = JSON.stringify(e.message);
      error = true;
    }

    fs.appendFileSync(
      "./data/" + argv.id,
      JSON.stringify({
        s: string,
        t: Date.now() - before,
        r: message,
        d: Date.now(),
        e: error
      }) + "\n"
    );
  }

  if (details) {
    return { result, message, statements, time: Date.now() - before, error };
  } else {
    if (error) {
      throw result;
    }

    return result;
  }
};
