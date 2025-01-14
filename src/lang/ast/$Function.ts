import $Node from "./$Node";
import _ from "lodash";
import $Identifier from "./$Identifier";
import { Function } from "acorn";

class $Function extends $Node {
  resolve(scope) {
    if (scope) {
      const cloned = _.cloneDeep(this.node);
      scope.callback = cloned.params.map((param) => new $Identifier(param));
      resolveNode(scope, cloned.body);
      scope.callback = [];
      return cloned;
    } else {
      return this.node;
    }
  }

  graph(scope) {
    const node = this.node as Function;
    scope.callback = node.params.map((param) => new $Identifier(param));
    const list = graphNode(scope, node.body);
    scope.callback = [];
    return list;
  }
}

function resolveNode(scope, node) {
  switch (node.type) {
    case "VariableDeclaration": {
      node.declarations.forEach((declaration) => {
        resolveNode(scope, declaration.init);
      });
      break;
    }
    case "BlockStatement": {
      node.body.forEach((statement) => resolveNode(scope, statement));
      break;
    }
    case "ExpressionStatement": {
      resolveNode(scope, node.expression);
      break;
    }
    case "AssignmentExpression": {
      resolveNode(scope, node.left);
      resolveNode(scope, node.right);
      break;
    }
    case "IfStatement": {
      resolveNode(scope, node.test);
      resolveNode(scope, node.consequent);
      if (node.alternate) {
        resolveNode(scope, node.alternate);
      }
      break;
    }
    case "ReturnStatement": {
      resolveNode(scope, node.argument);
      break;
    }
    default: {
      resolveIdentifier(scope, node);
    }
  }
}

function resolveIdentifier(scope, node) {
  if (["BinaryExpression", "LogicalExpression"].includes(node.type)) {
    resolveIdentifier(scope, node.left);
    resolveIdentifier(scope, node.right);
  } else {
    if (["Identifier", "MemberExpression"].includes(node.type)) {
      const identifier = new $Identifier(_.cloneDeep(node));
      const resolved = identifier.resolve(scope);

      Object.keys(node).forEach((key) => delete node[key]);
      Object.assign(node, resolved);
    }
  }
}

function graphNode(scope, node) {
  switch (node.type) {
    case "VariableDeclaration": {
      node.declarations.forEach((declaration) => {
        graphNode(scope, declaration.init);
      });
      break;
    }
    case "BlockStatement": {
      node.body.forEach((statement) => graphNode(scope, statement));
      break;
    }
    case "ExpressionStatement": {
      graphNode(scope, node.expression);
      break;
    }
    case "AssignmentExpression": {
      graphNode(scope, node.left);
      graphNode(scope, node.right);
      break;
    }
    case "IfStatement": {
      graphNode(scope, node.test);
      graphNode(scope, node.consequent);
      if (node.alternate) {
        graphNode(scope, node.alternate);
      }
      break;
    }
    case "ReturnStatement": {
      graphNode(scope, node.argument);
      break;
    }
    default: {
      return graphIdentifier(scope, node);
    }
  }

  return [];
}

function graphIdentifier(scope, node, acc: $Identifier[] = []): $Identifier[] {
  if (["BinaryExpression", "LogicalExpression"].includes(node.type)) {
    graphIdentifier(scope, node.left, acc);
    graphIdentifier(scope, node.right, acc);
  } else {
    if (["Identifier", "MemberExpression"].includes(node.type)) {
      const identifier = new $Identifier(node);
      const graph = identifier.graph(scope);

      if (graph.length) {
        acc.push(...graph);
      }
    }
  }

  return acc;
}

export default $Function;
