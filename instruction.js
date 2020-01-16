module.exports = class Instruction {
  constructor(scope, statement, before, run, graph) {
    this.scope = scope;
    this.statement = statement;
    this.before = before;
    this.run = run;
    this.graph = graph;
  }
};
