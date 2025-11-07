class Instruction {
  scope: object;
  statement: object;
  before: (() => void) | null;
  run: (() => void) | null;
  graph: (() => void) | null;
  after: (() => void) | null;
  derivative: boolean;
  priority: boolean;

  constructor(
    scope: object,
    statement: object,
    before: (() => void) | null,
    run: (() => void) | null,
    graph: (() => void) | null,
    after: (() => void) | null,
    derivative: boolean = true,
    priority: boolean = false
  ) {
    this.scope = scope;
    this.statement = statement;
    this.before = before;
    this.run = run;
    this.graph = graph;
    this.after = after;
    this.derivative = derivative;
    this.priority = priority;
  }
}

export default Instruction;
