import $CALL from "../$nuc/$CALL";
import $Identifier from "./$Identifier";
import $Node from "./$Node";
import Expression from "../../Expression";
import FUNCTION from "../../nuc/FUNCTION";
import { Node } from "acorn";
import _ from "lodash";
import { process } from "../../stack";
import { retrieve } from "../../graph";
import serialize from "../../lib/serialize";

class $Call extends $Node {
  get first(): $Identifier | null {
    const callee = rootCallee(this.node);
    if (!callee) return null;

    const identifier = new $Identifier(callee as unknown as Node);
    return identifier.first;
  }

  get object(): $Identifier | null {
    const callee = rootCallee(this.node);
    if (!callee) return null;

    const identifier = new $Identifier(callee as unknown as Node);
    return identifier.object;
  }

  get last(): $Identifier | null {
    const root = rootCallee(this.node);
    if (!root) return null;

    const identifier = new $Identifier(root as unknown as Node);
    return identifier.last;
  }

  get function(): $Identifier {
    const root = rootCallee(this.node);
    if (!root) {
      throw new Error("Function callee is null or undefined");
    }
    return new $Identifier(root as unknown as Node);
  }

  resolve(scope: unknown): Node {
    if (scope) {
      const clone = _.cloneDeep(this);
      const first = clone.first;

      if (first && retrieve(first) instanceof FUNCTION) {
        const { value } = process(
          [$CALL((this.node as any).callee, (this.node as any).arguments)],
          scope
        );
        const json = serialize(value, "state");
        const astNode = JSON.parse(json);
        const expression = new Expression(astNode);
        return expression.resolve(scope) as Node;
      } else {
        if (first && first.type !== "Literal") {
          const resolved = first.resolve(scope);
          const rootNode = findRoot(clone.node);

          if (resolved) {
            if (rootNode.object) {
              rootNode.object = resolved;
            } else {
              rootNode.callee = resolved;
            }
          }
        }

        resolveArguments(scope, clone.node);

        return clone.node as Node;
      }
    } else {
      return this.node as Node;
    }
  }

  graph(scope: unknown): $Node[] {
    const result: $Node[] = [];

    if (this.first) {
      const firstGraph = this.first.graph(scope);
      result.push(...firstGraph);
    }

    const argGraphs = traverseCallee(this.node, (callee) => {
      if (!callee.arguments) return [];

      return callee.arguments.flatMap((arg: Node | string) => {
        const node = $Node.convert(arg);
        return node ? node.graph(scope) : [];
      });
    });

    for (const graphs of argGraphs) {
      if (Array.isArray(graphs)) {
        for (const item of graphs) {
          if (item instanceof $Node) {
            result.push(item);
          }
        }
      }
    }

    return result;
  }

  walk(): $Node[] {
    const result: $Node[] = [];

    if (this.first) {
      const firstWalk = this.first.walk();
      result.push(...firstWalk);
    }

    const argWalks = traverseCallee(this.node, (callee) => {
      if (!callee.arguments) return [];

      return callee.arguments.flatMap((arg: Node | string) => {
        const node = $Node.convert(arg);
        return node ? node.walk() : [];
      });
    });

    for (const walks of argWalks) {
      if (Array.isArray(walks)) {
        for (const item of walks) {
          if (item instanceof $Node) {
            result.push(item);
          }
        }
      }
    }

    return result;
  }
}

function findRoot(node: Record<string, any>): Record<string, any> {
  let current = node;

  while (
    ["MemberExpression", "CallExpression"].includes(
      current.object?.type || current.callee?.type
    )
  ) {
    current = current.object || current.callee;
  }

  return current;
}

function rootCallee(node: Record<string, any>): Record<string, any> | null {
  let current = node;
  let callee = node.callee;

  if (!callee) return null;

  while (
    ["MemberExpression", "CallExpression"].includes(
      current.object?.type || current.callee?.type
    )
  ) {
    if (current.callee) {
      callee = current.callee;
    }

    current = current.object || current.callee;
  }

  return callee;
}

function traverseCallee<T = any>(
  node: Record<string, any>,
  func: (node: Record<string, any>) => T[]
): T[][] {
  let current = node;
  const acc: T[][] = [];

  while (
    ["MemberExpression", "CallExpression"].includes(
      current.type || current.object?.type || current.callee?.type
    )
  ) {
    if (current.callee) {
      acc.push(func(current));
    }

    current = current.object || current.callee;
  }

  return acc;
}

function resolveArguments(scope: unknown, node: Record<string, any>): void {
  let current = node;

  while (
    ["MemberExpression", "CallExpression"].includes(
      current?.type || current.object?.type
    )
  ) {
    if (current.callee && current.arguments) {
      current.arguments = current.arguments.map((arg: Node | string) => {
        const node = $Node.convert(arg);
        return node ? node.resolve(scope) : arg;
      });
    }

    current = current.callee || current.object;
  }
}

$Node.register("CallExpression", $Call);

export default $Call;

