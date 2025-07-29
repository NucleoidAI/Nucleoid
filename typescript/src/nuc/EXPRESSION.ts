import * as graph from "../graph";

import Call from "../lang/ast/Call";
import Evaluation from "../lang/Evaluation";
import Identifier from "../lang/ast/Identifier";
import NODE from "./NODE";
import type REFERENCE from "./REFERENCE";
import { append } from "../lang/estree/estree";
import { parse } from "../lang/estree/parser";
import serialize from "../lib/serialize";

interface StateModule {
  expression(scope: Scope, options: { value: string }): unknown;
  delete(scope: Scope, path: string): void;
  $: Record<string, Array<{ id: string }>>;
}

let state: StateModule;

setImmediate(() => {
  import("../state").then((module) => {
    state = module.default;
  });
});

interface Scope {
  root: Record<string, unknown>;
  retrieve(node: ExpressionNode): unknown;
  [key: string]: unknown;
}

interface MemberNode {
  toString(): string;
}

interface ExpressionNode {
  type: string;
  node: Record<string, unknown>;
  last?: MemberNode;
  object?: ExpressionNode;
  generate(scope: Scope): string;
  function?: {
    generate(scope: Scope): string;
  };
  first?: ExpressionNode;
}

interface ParsedNode {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any;
}

interface TokenIterable {
  left: ExpressionNode;
  right: ExpressionNode;
}

interface Tokens {
  map<T>(callback: (node: ExpressionNode) => T): T[];
  traverse(callback: (node: ExpressionNode) => string): string[];
  wrt?: boolean;
  graph(scope: Scope, callback: (node: ExpressionNode) => unknown): unknown;
  [Symbol.iterator](): Iterator<TokenIterable>;
}

interface GraphObject {
  next: Record<string, unknown>;
  value?: {
    link: {
      node: unknown;
    };
  };
}

class EXPRESSION {
  iof: string;
  tokens: Tokens;

  constructor(tokens: Tokens) {
    this.iof = this.constructor.name;
    this.tokens = tokens;
  }

  before(scope: Scope): void {
    this.tokens.map((node: ExpressionNode) => {
      if (
        node.type === "MemberExpression" &&
        node.last?.toString() === "value"
      ) {
        const value = state.expression(scope, {
          value: node.object!.generate(scope),
        });

        const newValue = parse(serialize(value, "state"), false);

        Object.keys(node.node).forEach((key) => delete node.node[key]);
        Object.assign(node.node, newValue);
      }
    });
  }

  run(scope: Scope, force = false): Evaluation | undefined {
    this.tokens.map((node: ExpressionNode) => {
      try {
        if (node.type === "CallExpression") {
          const func = state.expression(scope, {
            value: node.function!.generate(scope),
          });

          if (func?.value) {
            const value = state.expression(scope, {
              value: node.generate(scope),
            });
            const newNode = parse(serialize(value, "state"), false);

            Object.keys(node.node).forEach((key) => delete node.node[key]);
            Object.assign(node.node, newNode);
          }

          if (func?.write) {
            this.tokens.wrt = true;
          }
        }
      } catch (_err) {
        // Empty catch block - deliberate, preserving original behavior
      }
    });

    const expression = this.tokens.traverse((node: ExpressionNode) => {
      const evaluation = node.generate(scope);

      if (
        scope.retrieve(node) ||
        (node.type === "MemberExpression" && graph.retrieve(node.first))
      ) {
        try {
          const test = state.expression(scope, { value: evaluation });
          if (test === undefined) {
            return "undefined";
          }
        } catch (_error) {
          return "undefined";
        }
      }

      return evaluation;
    });

    if (force || !expression.includes("undefined")) {
      return new Evaluation(expression.join(""));
    }

    return undefined;
  }

  next(): unknown[] {
    return this.tokens.map((ast: ExpressionNode) => {
      if (ast instanceof Call) {
        const object = ast.object;

        if (object) {
          const node = graph.retrieve(object);

          if (node) {
            return Object.values(node.next);
          }
        }
      }

      return undefined;
    });
  }

  graph(scope: Scope): unknown {
    return this.tokens.graph(scope, (node: ExpressionNode) => {
      const retrieved = graph.retrieve(node);

      if (retrieved) {
        return retrieved;
      } else {
        let REFERENCE_Module: typeof REFERENCE;

        import("./REFERENCE").then((module) => {
          REFERENCE_Module = module.default;
        });

        for (const { left, right } of node) {
          const test = graph.retrieve(left) as GraphObject | undefined;

          if (test?.value && test.value instanceof REFERENCE_Module) {
            const link = new Identifier(
              append(test.value.link.node, right.node)
            );
            return graph.retrieve(link);
          }
        }

        const temporary = new NODE(node);
        // TODO NODE Direct
        graph.$[node as unknown as string] = temporary;
        return temporary;
      }
    });
  }
}

export default EXPRESSION;

