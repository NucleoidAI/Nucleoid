const CLASS = require("../../nuc/CLASS");
const $ = require("./$");
const Identifier = require("../ast/Identifier");

function build(name, methods = []) {
  const statement = new $CLASS();
  statement.nme = name;
  statement.mths = methods;
  return statement;
}

class $CLASS extends $ {
  run() {
    const name = new Identifier(this.nme);

    const statement = new CLASS(`$${name}`);
    statement.name = new Identifier(`$${name}`);
    statement.list = name;
    statement.methods = this.mths.reduce((acc, method) => {
      const identifier = method.nme;

      if (identifier.name === "constructor") {
        identifier.name = "$constructor";
      }

      acc[identifier.name] = method;
      return acc;
    }, {});

    return statement;
  }
}

module.exports = build;
