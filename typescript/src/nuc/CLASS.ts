import { $ALIAS } from "../lang/$nuc/$ALIAS";
import { $EXPRESSION } from "../lang/$nuc/$EXPRESSION";
import Evaluation from "../lang/Evaluation";
import NODE from "./NODE";
import _ from "lodash";
import graph from "../graph";
import state from "../state";

interface ExpressionNode {
  type: string;
  [key: string]: unknown;
}

class CLASS extends NODE {
  methods: unknown[];
  instances: Record<string, unknown>;
  declarations: Record<string, unknown>;
  name!: {
    node?: unknown;
    toString(): string;
  };
  list!: { node?: unknown };
  destroyed?: boolean;

  constructor(key: string) {
    super(key);
    this.methods = [];
    this.instances = {};
    this.declarations = {};
  }

  run(scope: Record<string, unknown>): NODE | NODE[] | null {
    const cls = graph.retrieve(this.name);

    if (cls) {
      if (_.isEqual(this.methods, cls.methods)) {
        this.destroyed = true;
        return null;
      }
    }

    state.assign(scope, this.name, new Evaluation(`class ${this.name}{}`));

    const list: NODE[] = [];

    if (!cls) {
      state.call(scope, "classes.push", [`state.${this.name}`]);

      const empty: ExpressionNode = { type: "ArrayExpression", elements: [] };
      // Use unknown as intermediate type to avoid direct type conversion errors
      const alias = $ALIAS(
        this.name.node as unknown,
        this.list.node as unknown,
        empty as unknown
      ) as unknown as NODE;
      list.push(alias);
    }

    list.push(
      $EXPRESSION({
        type: "Literal",
        value: null,
        raw: "null",
      } as unknown) as unknown as NODE
    );

    return list;
  }

  beforeGraph(): { destroyed: boolean } | undefined {
    if (this.destroyed) {
      return { destroyed: true };
    }

    const cls = graph.retrieve(this.key);

    if (cls instanceof CLASS) {
      this.declarations = cls.declarations;
    }
  }
}

export default CLASS;

