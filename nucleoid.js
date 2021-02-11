var Stack = require("./stack");
var Statement = require("./statement");
const fs = require("fs");
const argv = require("yargs").argv;
var Message = require("./message");
var Event = require("./event");
var transaction = require("./transaction");
var Macro = require("./macro");

module.exports.run = function(string, details, cacheOnly) {
  let before = Date.now();
  let error, json;

  let s = Macro.apply(string);

  try {
    var statements = Statement.compile(s);
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
      json = JSON.stringify(`${result.constructor.name}: ${result.message}`);
    } else {
      json = JSON.stringify(result);
    }
  } catch (e) {
    json = JSON.stringify(e.message);
    error = true;
  }

  let messages = Message.list();
  let events = Event.list();

  Message.clear();
  Event.clear();

  let date = Date.now();
  let time = date - before;

  if (argv.id !== undefined && argv.path !== undefined && !cacheOnly) {
    fs.appendFileSync(
      `${argv.path}/${argv.id}`,
      JSON.stringify({
        s,
        t: time,
        r: json,
        d: date,
        e: error,
        m: messages,
        v: events
      }) + "\n"
    );
  }

  if (details) {
    return {
      string: s,
      result: json,
      statements,
      date,
      time,
      error,
      messages,
      events
    };
  } else {
    if (error) {
      throw result;
    }

    return result;
  }
};
