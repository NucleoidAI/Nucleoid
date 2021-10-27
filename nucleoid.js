const Stack = require("./stack");
const Statement = require("./statement");
const fs = require("fs");
const argv = require("yargs").argv;
const Message = require("./message");
const Event = require("./event");
const transaction = require("./transaction");
const Macro = require("./macro");

module.exports.run = function (statement, config) {
  config = config || {};

  {
    const { declarative } = config;
    config.declarative = declarative === undefined ? true : declarative;
  }

  const { details, cacheOnly, declarative } = config;

  let before = Date.now();
  let statements, result, error, json, execs;

  let s = Macro.apply(statement);

  try {
    statements = Statement.compile(s);
    transaction.start();
    result = Stack.process(statements, config);
    execs = transaction
      .end()
      .filter((t) => t.exec)
      .map((t) => t.exec);
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
        c: declarative,
        t: time,
        r: json,
        d: date,
        e: error,
        m: messages,
        v: events,
        x: execs && execs.length ? execs : undefined,
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
      events,
      execs,
    };
  } else {
    if (error) {
      throw result;
    }

    return result;
  }
};
