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

module.exports.process = function (statement, options = {}) {
  if (!process) {
    console.error("The runtime has not started");
    require("process").exit(1);
  }

  _options = { ...process.options(), ...options };
  const { declarative, cacheOnly, details } = _options;

  let before = Date.now();
  let statements, result, error, execs;

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

  let messages = Message.list();
  let events = Event.list();

  Message.clear();
  Event.clear();

  let date = Date.now();
  let time = date - before;

  if (!cacheOnly) {
    if (result instanceof Error)
      result = `${result.constructor.name}: ${result.message}`;

    fs.appendFileSync(
      `${path}/${argv.id || "main"}`,
      JSON.stringify({
        s,
        c: declarative ? true : undefined,
        t: time,
        r: result,
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
    if (result instanceof Error)
      result = `${result.constructor.name}: ${result.message}`;

    return {
      result,
      date,
      time,
      error,
      messages,
      events,
    };
  } else {
    if (error) {
      throw result;
    }

    return result;
  }
};

module.exports.options = () => _options;
