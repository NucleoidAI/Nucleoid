var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");

module.exports = class EXPRESSION {
  run(local, instance) {
    let tokens = this.tokens.map(token => {
      let parts = token.split(".");

      if (instance && parts[0] == instance.class.name) {
        parts[0] = instance.variable;
        token = parts.join(".");
      }

      if (graph.node[parts[0]]) {
        return "state." + token;
      } else if (local[token]) {
        let value = local[token];

        if (typeof value == "string") {
          return '"' + value + '"';
        } else {
          return value;
        }
      } else {
        return token;
      }
    });

    return eval(tokens.join(""));
  }
};
