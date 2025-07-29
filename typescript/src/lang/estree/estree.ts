import _ from "lodash";

export interface Node {
  type: string;
  [key: string]: any;
}

export interface MemberExpression extends Node {
  type: "MemberExpression";
  object?: Node;
  property?: Node;
  computed: boolean;
}

export interface Identifier extends Node {
  type: "Identifier";
  name: string;
}

export type ExpressionNode = MemberExpression | Identifier | Node;

export function root(node: ExpressionNode): ExpressionNode {
  let current = node as MemberExpression;
  while (
    current.object &&
    !["Identifier", "ThisExpression", "Literal"].includes(current.object.type)
  ) {
    current = current.object as MemberExpression;
  }
  return current;
}

export function append(
  source: ExpressionNode | null | undefined,
  target: ExpressionNode | null | undefined
): ExpressionNode {
  const clonedSource = source ? _.cloneDeep(source) : null;
  const clonedTarget = target ? _.cloneDeep(target) : null;

  if (!clonedSource && !clonedTarget) {
    throw new Error("Cannot append: both source and target are empty");
  }
  if (!clonedSource) {
    return clonedTarget!;
  }
  if (!clonedTarget) {
    return clonedSource;
  }

  const empty: MemberExpression = {
    type: "MemberExpression",
    computed: false,
    object: clonedSource,
    property: undefined as any,
    start: (clonedSource as any).start,
    end: (clonedTarget as any).end,
  };

  if (clonedTarget.type === "Identifier") {
    empty.property = clonedTarget;
    return empty;
  }

  if (clonedTarget.type === "MemberExpression") {
    const firstRoot = root(
      clonedTarget as MemberExpression
    ) as MemberExpression;
    empty.property = firstRoot.object!;
    firstRoot.object = empty;
    return clonedTarget as ExpressionNode;
  }

  throw new Error("Invalid expression type in append()");
}

