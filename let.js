var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");

module.exports = class LET {
  run(local) {
    let list = this.expression.tokens.map(token =>
      graph.node[token.split(".")[0]] ? "state." + token : token
    );

    local[this.variable] = eval(list.join(""));
  }

  graph() {}
};
