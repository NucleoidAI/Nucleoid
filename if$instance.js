var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

module.exports = class IF$INSTANCE {
  run() {
    if (this.function && this.function(state)) return this.true;

    const tokens = this.condition.tokens.map(token => {
      let parts = token.split(".");
      if (parts[0] == this.class) parts[0] = this.instance;
      return parts.join(".");
    });

    const condition = { tokens: tokens };
    const key = "if(" + condition.tokens.join("") + ")";
    graph.node[key] = new Node(this);

    condition.tokens.forEach(token => {
      if (graph.node[token]) graph.node[token].edge[key] = graph.node[key];
    });

    let list = ["return "].concat(
      condition.tokens.map(token =>
        graph.node[token.split(".")[0]] ? "state." + token : token
      )
    );

    this.function = new Function("state", list.join(""));
    if (this.function(state)) return this.true;
  }
};
