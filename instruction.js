module.exports = class Instruction {
  constructor(scope, statement, graph) {
    this.scope = scope;
    this.statement = statement;

    if (graph == undefined) {
      this.graph = true;
    } else {
      this.graph = graph;
    }
  }
};
