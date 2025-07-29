import * as transaction from "../transaction";

import { $ } from "../graph";

let sequence = 0;

class NODE {
  key: string;
  next: any;
  previous: any;
  sequence: number;

  constructor(key) {
    this.key = key.toString();
    this.next = {};
    this.previous = {};
    this.sequence = sequence++;
  }

  before(scope?) {}
  run(scope?): any {}
  beforeGraph(scope?) {}
  graph(scope?) {}
  after(scope?) {}

  // TODO Move this to graph
  static register(key, node) {
    transaction.register($, key, node);
  }

  static replace(sourceKey, targetNode) {
    transaction.register(targetNode.block, $[sourceKey].block);

    for (let node in $[sourceKey].next) {
      transaction.register(targetNode.next, node, $[sourceKey].next[node]);
      transaction.register($[sourceKey].next, node, undefined);
    }

    for (let node in $[sourceKey].previous) {
      transaction.register($[node].next, sourceKey, undefined);
    }

    transaction.register($, sourceKey, targetNode);
  }

  static direct(sourceKey: string, targetKey: string, targetNode: NODE): void {
    transaction.register($[sourceKey].next, targetKey, targetNode);
    transaction.register(targetNode.previous, sourceKey, $[sourceKey]);
  }
}

export default NODE;

