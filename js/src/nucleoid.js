const runtime = require("./runtime");
const context = require("./context");
const process = require("./process");
const _config = require("./config");
const { parseFunction } = require("./lang/estree/parser");
const { generate } = require("./lang/estree/generator");
const chalk = require("chalk");

function start(config = {}) {
  _config.init(config);

  console.log("🌿 " + chalk.greenBright("Nucleoid") + " runtime is started");
  console.log(chalk.blueBright("🌎 Inspired by Nature\n"));

  process.init();

  return new Promise((resolve) => {
    setImmediate(() => {
      context.run();
      resolve();
    });
  });
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
    const { params, body } = parseFunction(statement.toString());
    scope =
      scope && params.length
        ? `let ${generate(params[0])}=${JSON.stringify(scope)};`
        : "";
    return runtime.process(`${scope}${generate(body)}`, options);
  }
}

module.exports.start = start;
module.exports.register = register;
module.exports.run = run;
