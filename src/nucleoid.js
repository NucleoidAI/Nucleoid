const runtime = require("./runtime");
const parser = require("./lib/parser");
const context = require("./lib/context");
const terminal = require("./terminal");
const process = require("./process");
const _config = require("./config");

function start(config = {}) {
  const { port, test } = _config.init(config);
  process.init();
  context.run();

  if (!test) {
    terminal.listen(port.terminal);
  }
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
