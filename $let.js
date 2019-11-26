var $ = require("./$");
var graph = require("./graph");
var CLASS = require("./class");
var LET = require("./let");
var LET$CLASS = require("./let$class");
var OBJECT = require("./object");
var LET$OBJECT = require("./let$object");
var EXPRESSION = require("./expression");
var REFERENCE = require("./reference");
var Local = require("./local");
var Identifier = require("./identifier");

module.exports = function(name, value) {
  let statement = new $LET();
  statement.name = name;
  statement.value = value;
  return statement;
};

class $LET extends $ {
  run(scope) {
    let parts = Identifier.splitLast(this.name);

    if (parts[1] !== undefined && !Local.check(scope, parts[1])) {
      throw ReferenceError(`'${parts[1]}' is not defined`);
    }

    parts = this.name.split(".");
    if (parts.length > 1 && parts[parts.length - 1] === "value")
      throw new TypeError();

    let value = this.value.run();

    if (value instanceof EXPRESSION) {
      for (let token of value.tokens) {
        let prefix = token.split(".")[0];

        if (graph[prefix] && graph[prefix] instanceof CLASS) {
          let statement = new LET$CLASS();
          statement.class = graph[prefix];
          statement.name = this.name;
          statement.value = value;
          return statement;
        }
      }

      let statement = new LET();
      statement.name = this.name;
      statement.value = value;
      return statement;
    } else if (value instanceof REFERENCE) {
      let statement = new LET();
      statement.name = this.name;
      statement.value = value;
      return statement;
    } else if (value instanceof OBJECT) {
      let statement = new LET$OBJECT();
      statement.name = this.name;
      statement.class = value.class;
      return statement;
    }
  }
}
