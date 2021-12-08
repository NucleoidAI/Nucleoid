const stack = require("./stack");
const Statement = require("./statement");
const fs = require("fs");
const argv = require("yargs").argv;
const Message = require("./message");
const Event = require("./event");
const transaction = require("./transaction");
const Macro = require("./macro");
const File = require("./file");
const path = File.data;

let process;
let _options = {};

setImmediate(() => {
  process = require("./process");
});

module.exports.process = function (statement, options) {
  options = options || {};

  const { declarative } =
    options.declarative === undefined ? process.options() : options;
  const { details } =
    options.details === undefined ? process.options() : options;
  const { cacheOnly } =
    options.cacheOnly === undefined ? process.options() : options;

  _options = { declarative, details, cacheOnly };

  let before = Date.now();
  let statements, result, error, json, execs;

  let s = Macro.apply(statement, _options);

  try {
    statements = Statement.compile(s, _options);
    transaction.start();
    result = stack.process(statements, null);
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

  if (!cacheOnly) {
    fs.appendFileSync(
      `${path}/${argv.id || "main"}`,
      JSON.stringify({
        s,
        c: declarative ? true : undefined,
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

  _options = {};

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

module.exports.options = () => _options;
