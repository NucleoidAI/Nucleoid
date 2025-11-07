import * as graph from "../graph";
import * as state from "../state";

import DELETE from "./DELETE";
import Instruction from "../Instruction";

interface Node {
  key: string;
  name: string;
  properties: Record<string, unknown>;
  object?: {
    properties: Record<string, unknown>;
  };
  class?: {
    list: {
      toString(): string;
    };
  };
  previous: Record<string, unknown>;
  next: Record<string, unknown>;
}

interface Scope {
  root: Record<string, unknown>;
  [key: string]: unknown;
}

interface StateType {
  $: Record<string, Array<{ id: string }>>;
  delete(scope: Scope, path: string): void;
}

class DELETE$OBJECT extends DELETE {
  run(scope: Scope): { next?: Instruction[]; value: boolean } {
    const node: Node = graph.retrieve(this.variable!.key);
    const name: string = node.name;

    if (Object.keys(node.properties).length > 0)
      throw ReferenceError(`Cannot delete object '${this.variable!.key}'`);

    if (node.object) {
      delete node.object.properties[name];
    } else {
      const list: string = node.class!.list.toString();
      state.delete(scope, `${list}.${name}`);

      const index: number = (state as unknown as StateType).$[list].findIndex(
        (object: { id: string }) => object.id === node.key
      );
      (state as unknown as StateType).$[list].splice(index, 1);
    }

    return super.run(scope);
  }

  graph(): void {
    const node: Node = graph.retrieve(this.variable!.key);

    for (const key in node.previous) {
      delete node.next[key];
    }

    delete graph.$[node.key];
  }
}

export default DELETE$OBJECT;
