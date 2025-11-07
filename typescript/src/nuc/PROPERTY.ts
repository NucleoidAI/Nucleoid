import * as estree from "../lang/estree/estree";
import * as graph from "../graph";
import * as state from "../state";

import Identifier from "../lang/ast/Identifier";
import NODE from "./NODE";
import REFERENCE from "./REFERENCE";

class PROPERTY extends NODE {
  object!: { value: any; properties: Record<string, PROPERTY> };
  name!: Identifier;
  value!: {
    before: (scope: any) => void;
    run: (scope: any) => any;
    graph: (scope: any) => any;
  };

  before(scope: any): void {
    this.value.before(scope);
  }

  run(scope: any): { value: any } | undefined {
    const evaluation = this.value.run(scope);

    const ref = this.object.value;
    const target =
      ref instanceof REFERENCE
        ? graph.retrieve((ref as any).link)
        : this.object;

    const appended = estree.append(target.resolve().node, this.name.node);
    const variable = new Identifier(appended);

    if (!evaluation) {
      state.delete(scope, variable.toString());
      return;
    }

    const assigned = state.assign(scope, variable.toString(), evaluation);
    return { value: assigned };
  }

  graph(scope: any): any {
    const ref = this.object.value;
    const target =
      ref instanceof REFERENCE
        ? graph.retrieve((ref as any).link)
        : this.object;

    target.properties[this.name.toString()] = this;
    return this.value.graph(scope);
  }
}

export default PROPERTY;
