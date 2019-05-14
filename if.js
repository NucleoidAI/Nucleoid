var Statement = require("./statement");
var graph = require("./state").state.graph;
var Node = require("./state").state.Node;
var crypto = require("crypto");

module.exports = class If extends Statement {
  constructor(statement) {
    super(statement.string);
    this.tokens = statement.tokens;
    this.token = statement.token;
    this.offset = statement.offset;
  }

  run() {
    this.next();

    let nodes = [];
    let expression = this.each(function(token) {
      if (graph[token]) {
        nodes.push(token);
        return true;
      } else {
        return false;
      }
    }, ")");

    let shasum = crypto
      .createHash("sha1")
      .update("if(" + expression + ")")
      .digest("hex");

    graph[shasum] = new Node();
    graph[shasum].data = { statement: this };
    nodes.forEach(e => (graph[e].nodes[shasum] = graph[shasum]));

    this.expression = expression;
    this.key = shasum;
    this.next();
    this.trueBlock = this.nextBlock();
  }
};
