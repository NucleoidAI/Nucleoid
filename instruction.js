module.exports = class Instruction {
  constructor(scope, statement, run, graph) {
    this.scope = scope;
    this.statement = statement;

    if (run == undefined) {
      this.run = true;
    } else {
      this.run = run;
    }

    if (graph == undefined) {
      this.graph = true;
    } else {
      this.graph = graph;
    }
  }
};
