var Stack = require("./stack");
var Statement = require("./statement");
const fs = require("fs");
const argv = require("yargs").argv;
var Message = require("./message");
var transaction = require("./transaction");

module.exports.run = function(string, details, cacheOnly) {
  let before = Date.now();
  let error, json;

  try {
    var statements = Statement.compile(string);
    transaction.start();
    var result = Stack.process(statements);
    transaction.end();
  } catch (e) {
    transaction.rollback();
    result = e;
    error = true;
  }

  try {
    if (result instanceof Error) {
      json = JSON.stringify(result.message);
    } else {
      json = JSON.stringify(result);
    }
  } catch (e) {
    json = JSON.stringify(e.message);
    error = true;
  }

  let messages = Message.list();
  Message.clear();

  let date = Date.now();
  let time = date - before;

  if (argv.id !== undefined && !cacheOnly) {
    fs.appendFileSync(
      "./data/" + argv.id,
      JSON.stringify({
        s: string,
        t: time,
        r: json,
        d: date,
        e: error,
        m: messages
      }) + "\n"
    );
  }

  if (details) {
    return { result: json, statements, date, time, error, messages };
  } else {
    if (error) {
      throw result;
    }

    return result;
  }
};
