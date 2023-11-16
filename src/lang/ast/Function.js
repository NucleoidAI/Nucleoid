const Node = require("./Node");
const _ = require("lodash");
const Identifier = require("./Identifier");

class Function extends Node {
  resolve(scope) {
    if (scope) {
      const cloned = _.cloneDeep(this.node);
      cloned.params.forEach((param) =>
        scope.callback.push(new Identifier(param))
      );

      resolveNode(scope, cloned.body);
      return cloned;
    } else {
      return this.node;
    }
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
      traverse(scope, node);
    }
  }
}

function traverse(scope, node) {
  if (["BinaryExpression", "LogicalExpression"].includes(node.type)) {
    traverse(scope, node.left);
    traverse(scope, node.right);
  } else {
    if (["Identifier", "MemberExpression"].includes(node.type)) {
      const identifier = new Identifier(node);
      const resolved = identifier.resolve(scope);

      for (const key in resolved) {
        node[key] = resolved[key];
      }
    }
  }
}

module.exports = Function;
