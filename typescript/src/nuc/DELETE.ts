import Instruction from "../Instruction";
import NODE from "./NODE";
import graph from "../graph";
import state from "../state";

interface Scope {
  root: Record<string, unknown>;
  [key: string]: unknown;
}

interface Variable {
  key: string;
}

class DELETE {
  variable?: Variable;

  before(): void {}

  run(scope: Scope): { next?: Instruction[]; value: boolean } {
    if (this.variable) {
      const key = this.variable.key;
      state.delete(scope, key);

      const list: Instruction[] = [];

      for (const node in graph.retrieve(key).next) {
        const statement = graph.retrieve(node);
        list.push(
          new Instruction(scope.root, statement, false, true, false, false)
        );
      }

      return { next: list, value: true };
    } else {
      return { value: false };
    }
  }

  beforeGraph(): void {}

  graph(): void {
    if (!this.variable) {
      return;
    }

    const node = graph.retrieve(this.variable.key);

    for (const key in node.previous) {
      delete node.next[key];
    }

    const empty = new NODE(node.key);

    for (const key in node.next) {
      empty.next[key] = node.next[key];
      delete node.next[key];
    }

    const name = node.name;
    delete node.object.properties[name];
    delete graph.$[node.key];
    graph.$[node.key] = empty;
  }
}

export default DELETE;
