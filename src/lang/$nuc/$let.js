const $ = require("./$");
const graph = require("../../graph");
const CLASS = require("../../class");
const LET = require("../../let");
const LET$CLASS = require("../../let$class");
const OBJECT = require("../../object");
const LET$OBJECT = require("../../let$object");
const EXPRESSION = require("../../expression");
const REFERENCE = require("../../reference");
const Local = require("../../lib/local");
const Id = require("../../lib/identifier");

function construct(name, value, constant) {
  let statement = new $LET();
  statement.name = name;
  statement.value = value;
  statement.constant = constant;
  return statement;
}

class $LET extends $ {
  run(scope) {
    let parts = Id.splitLast(this.name);

    if (parts[1] !== undefined && !Local.check(scope, parts[1])) {
      throw ReferenceError(`${parts[1]} is not defined`);
    }

    parts = this.name.split(".");
    if (parts.length > 1 && parts[parts.length - 1] === "value")
      throw TypeError("Cannot use 'value' in local");

    let value = this.value.run();

    if (value instanceof EXPRESSION) {
      for (let token of value.tokens.list()) {
        let prefix = token.split(".")[0];

        if (graph[prefix] && graph[prefix] instanceof CLASS) {
          let statement = new LET$CLASS();
          statement.class = graph[prefix];
          statement.name = this.name;
          statement.value = value;
          statement.constant = this.constant;
          return statement;
        }
      }

      let statement = new LET();
      statement.name = this.name;
      statement.value = value;
      statement.constant = this.constant;
      return statement;
    } else if (value instanceof REFERENCE) {
      let statement = new LET();
      statement.name = this.name;
      statement.value = value;
      statement.constant = this.constant;
      return statement;
    } else if (value instanceof OBJECT) {
      let object = new OBJECT();
      object.class = value.class;
      object.args = value.args;

      let statement = new LET$OBJECT();
      statement.name = this.name;
      statement.object = object;
      statement.constant = this.constant;

      return [object, statement];
    }
  }
}

module.exports = construct;
