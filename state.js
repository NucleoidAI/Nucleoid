var state = { graph: {} };

state.Node = class Node {
  constructor() {
    this.data = {};
    this.nodes = {};
  }
};

module.exports.state = state;
