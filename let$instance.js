var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");

module.exports = class LET$INSTANCE {
  run(local) {
    const tokens = this.expression.tokens.map(token => {
      let parts = token.split(".");
      if (parts[0] == this.class) parts[0] = this.instance;
      return parts.join(".");
    });

    const expression = { tokens: tokens };

    let list = expression.tokens.map(token =>
      graph.node[token.split(".")[0]] ? "state." + token : token
    );

    local[this.variable] = eval(list.join(""));
  }

  graph() {}
};
