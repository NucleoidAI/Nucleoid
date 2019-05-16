var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

module.exports = class ASSIGNMENT$INSTANCE {
  run(local) {
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

    let list = expression.tokens.map(token => {
      if (graph.node[token.split(".")[0]]) {
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

    this.function = new Function(
      "state",
      "state." + variable + "=" + list.join("")
    );
    this.function(state);
  }

  graph() {
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
  }
};
