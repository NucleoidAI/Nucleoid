var state = require("./state");
var graph = require("./graph");
var Node = require("./node");

module.exports = class IF {
  run() {
    if (this.function && this.function(state)) return this.true;

    let list = ["return "].concat(
      this.condition.tokens.map(token =>
        graph.node[token.split(".")[0]] ? "state." + token : token
      )
    );

    this.function = new Function("state", list.join(""));
    if (this.function(state)) return this.true;
  }

  graph() {
    const condition = this.condition;
    const key = "if(" + condition.tokens.join("") + ")";
    graph.node[key] = new Node(this);

    condition.tokens.forEach(token => {
      if (graph.node[token]) graph.node[token].edge[key] = graph.node[key];
    });
  }
};
