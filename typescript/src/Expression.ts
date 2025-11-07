import Node from "./lang/ast/Node";

type BinaryExpression = { type: "BinaryExpression"; left: ASTNode; right: ASTNode; operator: string };
type LogicalExpression = { type: "LogicalExpression"; left: ASTNode; right: ASTNode; operator: string };
type UnaryExpression = { type: "UnaryExpression"; argument: ASTNode; operator: string };
type ASTNode = BinaryExpression | LogicalExpression | UnaryExpression | { type: string; [key: string]: unknown };

// TODO Organize expression structure
class Expression extends Node {
  node: ASTNode;
  constructor(node: ASTNode) {
    super();
    this.node = node;
  }

  map<T>(fn: (ast: ASTNode) => T): T[] {
    return mapReduce(this.node, fn);
  }

  find<T>(fn: (ast: ASTNode) => T): T | undefined {
    // TODO Optimize this with own reduce
    const [first] = mapReduce(this.node, fn);
    return first;
  }

  resolve(scope: object): unknown {
    return Node.convert(this.node).resolve(scope);
  }

  traverse<T>(fn: (ast: ASTNode) => T): T[] {
    return traverseReduce(this.node, fn);
  }

  graph<T>(scope: object, fn: (ast: ASTNode) => T): T[] {
    return graphReduce(scope, this.node, fn);
  }
}

// TODO Traverse with different types
function traverseReduce<T>(exp: ASTNode, fn: (ast: ASTNode) => T, acc: T[] = []): T[] {
  if (exp.type === "BinaryExpression") {
    if (exp.left.type === "BinaryExpression") {
      acc.push("(");
    }

    traverseReduce(exp.left, fn, acc);

    if (exp.left.type === "BinaryExpression") {
      acc.push(")");
    }

    acc.push(exp.operator);

    if (exp.right.type === "BinaryExpression") {
      acc.push("(");
    }

    traverseReduce(exp.right, fn, acc);

    if (exp.right.type === "BinaryExpression") {
      acc.push(")");
    }
  } else if (["LogicalExpression"].includes(exp.type)) {
    traverseReduce(exp.left, fn, acc);
    acc.push(exp.operator);
    traverseReduce(exp.right, fn, acc);
  } else if (exp.type === "UnaryExpression") {
    acc.push(`${exp.operator} `);
    traverseReduce(exp.argument, fn, acc);
  } else {
    const ast = Node.convert(exp);
    const curr = fn(ast);

    if (curr) {
      acc.push(curr);
    }
  }

  return acc.flat(Infinity);
}

function mapReduce<T>(exp: ASTNode, fn: (ast: ASTNode) => T, acc: T[] = []): T[] {
  if (["BinaryExpression", "LogicalExpression"].includes(exp.type)) {
    mapReduce(exp.left, fn, acc);
    mapReduce(exp.right, fn, acc);
  } else if (exp.type === "UnaryExpression") {
    mapReduce(exp.argument, fn, acc);
  } else {
    const ast = Node.convert(exp);
    const curr = fn(ast);

    if (curr) {
      acc.push(curr);
    }
  }

  return acc.flat(Infinity);
}

function graphReduce<T>(scope: object, exp: ASTNode, fn: (ast: ASTNode) => T, acc: T[] = []): T[] {
  if (["BinaryExpression", "LogicalExpression"].includes(exp.type)) {
    graphReduce(scope, exp.left, fn, acc);
    graphReduce(scope, exp.right, fn, acc);
  } else if (exp.type === "UnaryExpression") {
    graphReduce(scope, exp.argument, fn, acc);
  } else {
    const ast = Node.convert(exp);

    const graphed = [ast.graph(scope)];
    graphed.flat(Infinity).forEach((item) => {
      if (item) {
        const curr = fn(item);

        if (curr) {
          acc.push(curr);
        }
      }
    });
  }

  return acc.flat(Infinity);
}

export default Expression;
