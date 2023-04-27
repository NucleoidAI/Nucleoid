const datastore = require("@nucleoidjs/datastore");
const stack = require("./stack");
const Statement = require("./statement");
const Message = require("./message");
const Event = require("./event");
const transaction = require("./transaction");
const Macro = require("./macro");
const config = require("./config");

module.exports.process = function (statement, options = {}) {
  options = { ...config(), ...options };
  const { declarative, details, cacheOnly, skipRead } = options;

  let before = Date.now();
  let statements, result, error, adjusts, write;

  let s = Macro.apply(statement, options);

  try {
    statements = Statement.compile(s, options);
    transaction.start();
    result = stack.process(statements, null, options);
    const list = transaction.end();
    adjusts = list.filter((t) => t.adjust).map((t) => t.adjust);
    write = !!list.length;
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

    if (write || !skipRead) {
      datastore.write({
        s,
        c: declarative ? true : undefined,
        t: time,
        r: result,
        d: date,
        e: error,
        m: messages,
        v: events,
        j: adjusts && adjusts.length ? adjusts : undefined,
        w: write ? write : undefined,
      });
    }
  }

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
