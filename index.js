const runtime = require("./src/runtime");
const parser = require("./src/libs/parser");
const context = require("./src/libs/context");
const terminal = require("./src/terminal");

function start(options = {}) {
  const process = require("./src/process");
  options = process.options(options);
  setImmediate(() => context.run());

  if (options.terminal !== false && options.test !== false) {
    terminal.listen(options.port);
  }

  console.log("Nucleoid runtime started");
}

function register(fn) {
  context.load({
    definition: fn.toString(),
    options: { declarative: true },
  });
}

function run(statement, p2, p3) {
  if (typeof statement === "string") {
    const options = p2;
    return runtime.process(`${statement}`, options);
  } else {
    let scope = p2;
    let options = p3;
    const { args, fn } = parser.fn(statement.toString());
    scope =
      scope && args.length ? `let ${args[0]}=${JSON.stringify(scope)};` : "";
    return runtime.process(`${scope}${fn}`, options);
  }
}

module.exports.start = start;
module.exports.register = register;
module.exports.run = run;
