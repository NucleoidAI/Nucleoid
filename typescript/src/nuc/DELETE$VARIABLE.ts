import * as graph from "../graph";

import DELETE from "./DELETE";

interface Node {
  key: string;
  previous: Record<string, unknown>;
  next: Record<string, unknown>;
}

class DELETE$VARIABLE extends DELETE {
  graph(): void {
    const node: Node = graph.retrieve(this.variable!.key);

    for (const key in node.previous) {
      delete node.next[key];
    }

    delete graph.$[node.key];
  }
}

export default DELETE$VARIABLE;
