const CLASS = require("../../nuc/CLASS");
const $ = require("./$");
const Identifier = require("../ast/Identifier");
const { append } = require("../estree/estree");

function build(name, methods = []) {
  const statement = new $CLASS();
  statement.name = name;
  statement.methods = methods;
  return statement;
}

class $CLASS extends $ {
  run() {
    const name = new Identifier(this.name);

    const statement = new CLASS(`$${name}`);
    statement.name = new Identifier(`$${name}`);
    statement.list = name;
    statement.methods = this.methods.map((method) => {
      method.name = append(statement.name.node, method.name);
      return method;
    });
    return statement;
  }
}

module.exports = build;
