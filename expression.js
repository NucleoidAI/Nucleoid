var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Local = require("./local");

class EXPRESSION {
  run(scope, instance) {
    let tokens = this.tokens.map(token => {
      if (token == "typeof") {
        return "typeof ";
      }

      let parts = token.split(".");

      if (instance && parts[0] == instance.class.name) {
        parts[0] = instance.name;
        token = parts.join(".");
      }

      let reference = Local.retrieve(scope, token);

      if (graph.node[parts[0]]) {
        return "state." + token;
      } else if (reference) {
        return reference;
      } else {
        return token;
      }
    });

    return eval(tokens.join(""));
  }
}

EXPRESSION.prototype.type = "REGULAR";
module.exports = EXPRESSION;
