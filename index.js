const runtime = require("./src/runtime");
const Token = require("./src/token");

const list = [];

const start = (options) => {
  const process = require("./src/process");
  process.options(options);

  setImmediate(() => {
    list.forEach(({ fn, options }) => run(fn.toString(), options));
  });
};

const register = (fn, options) => {
  list.push({ fn, options });
};

const run = (statement, p2, p3) => {
  if (typeof statement === "string") {
    const options = p2;
    return runtime.process(statement, options);
  } else {
    const scope = p2 || {};
    const options = p3;

    const string = statement.toString();
    let context = Token.next(string, 0);
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
    context = Token.nextBlock(string, context.offset);

    const exec =
      Object.entries(scope)
        .map(([key, value]) => `let ${key}=${JSON.stringify(value)};`)
        .join("") + context.block.trim();

    return runtime.process(exec, options);
  }
};

module.exports = { start, register, run };
