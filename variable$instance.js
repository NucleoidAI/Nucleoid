var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

module.exports = class VARIABLE$INSTANCE {
  run() {
    if (this.function) {
      this.function(state);
      return;
    }

    let variable = this.variable;
    let parts = this.variable.split(".");

    if (parts[0] == this.class) {
      parts[0] = this.instance;
      variable = parts.join(".");
    }

    const tokens = this.expression.tokens.map(token => {
      let parts = token.split(".");
      if (parts[0] == this.class) parts[0] = this.instance;
      return parts.join(".");
    });

    const expression = { tokens: tokens };
    graph.node[variable] = new Node(this);

    expression.tokens.forEach(token => {
      if (graph.node[token])
        graph.node[token].edge[variable] = graph.node[variable];
    });

    let list = expression.tokens.map(token =>
      graph.node[token.split(".")[0]] ? "state." + token : token
    );

    this.function = new Function(
      "state",
      "state." + variable + "=" + list.join("")
    );
    this.function(state);
  }
};
