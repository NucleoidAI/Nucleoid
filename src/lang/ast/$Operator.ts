import $Node from "./$Node";
import _ from "lodash";
import $Identifier from "./$Identifier";
import AST from "./convert";

class $Operator extends $Node {
  resolve(scope) {
    if (scope) {
      const cloned = _.cloneDeep(this.node);

      traverseResolve(scope, cloned);

      return cloned;
    } else {
      return this.node;
    }
  }

  walk() {
    return traverseWalk(this.node);
  }

  graph(scope): $Identifier[] {
    return traverseGraph(scope, this.node);
  }
}

function traverseWalk(node, acc: $Node[] = []): $Node[] {
  if (["BinaryExpression", "LogicalExpression"].includes(node.type)) {
    traverseWalk(node.left, acc);
    traverseWalk(node.right, acc);
  } else if (["UnaryExpression", "UpdateExpression"].includes(node.type)) {
    traverseWalk(node.argument, acc);
  } else {
    acc.push(...AST.convert(node).walk());
  }

  return acc;
}

function traverseResolve(scope, node) {
  if (["BinaryExpression", "LogicalExpression"].includes(node.type)) {
    traverseResolve(scope, node.left);
    traverseResolve(scope, node.right);
  } else if (["UnaryExpression", "UpdateExpression"].includes(node.type)) {
    traverseResolve(scope, node.argument);
  } else {
    const ast = AST.convert(_.cloneDeep(node));
    const resolved = ast.resolve(scope);

    Object.keys(node).forEach((key) => delete node[key]);
    Object.assign(node, resolved);
  }
}

function traverseGraph(scope, node, acc: $Identifier[] = []): $Identifier[] {
  if (["BinaryExpression", "LogicalExpression"].includes(node.type)) {
    traverseGraph(scope, node.left, acc);
    traverseGraph(scope, node.right, acc);
  } else if (["UnaryExpression", "UpdateExpression"].includes(node.type)) {
    traverseGraph(scope, node.argument, acc);
  } else {
    acc.push(...AST.convert(node).graph(scope));
  }

  return acc;
}

export default $Operator;
