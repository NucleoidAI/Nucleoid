module.exports = class Instruction {
  constructor(scope, statement, prepare, run, graph) {
    this.scope = scope;
    this.statement = statement;
    this.prepare = prepare;
    this.run = run;
    this.graph = graph;
  }
};
