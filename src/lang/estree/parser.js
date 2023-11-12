const acorn = require("acorn");
const $EXPRESSION = require("../$nuc/$EXPRESSION");
const $ASSIGNMENT = require("../$nuc/$ASSIGNMENT");
const $CLASS = require("../$nuc/$CLASS");
const $BLOCK = require("../$nuc/$BLOCK");
const $IF = require("../$nuc/$IF");
const $RETURN = require("../$nuc/$RETURN");
const $INSTANCE = require("../$nuc/$INSTANCE");
const $FUNCTION = require("../$nuc/$FUNCTION");

function parse(string, map = true) {
  const estree = acorn.parse(string, {
    ecmaVersion: 2020,
    allowReturnOutsideFunction: true,
  });

  return map ? estree.body.map(parseNode) : estree.body[0].expression;
}

function parseNode(node) {
  switch (node.type) {
    case "MemberExpression":
    case "Identifier": {
      return $EXPRESSION(node);
    }
    case "VariableDeclaration": {
      const {
        kind,
        declarations: [declaration],
      } = node;

      const { id, init } = declaration;

      return $ASSIGNMENT(kind.toUpperCase(), id, init);
    }
    case "ClassDeclaration": {
      const {
        id,
        body: { body },
      } = node;

      const methods = body.map((method) => parseNode(method));

      return $CLASS(id, methods);
    }
    case "AssignmentExpression": {
      const { left, right } = node;
      return $ASSIGNMENT(null, left, right);
    }
    case "BlockStatement": {
      const statements = node.body.map(parseNode);
      return $BLOCK(statements);
    }
    case "IfStatement": {
      const condition = node.test;
      const consequent = parseNode(node.consequent);
      const alternate = node.alternate ? parseNode(node.alternate) : null;

      return $IF(condition, consequent, alternate);
    }
    case "ExpressionStatement": {
      const { expression } = node;

      if (["AssignmentExpression", "NewExpression"].includes(expression.type)) {
        return parseNode(expression);
      } else {
        return $EXPRESSION(expression);
      }
    }
    case "NewExpression": {
      return $INSTANCE(node.callee, null, null, node.arguments);
    }
    case "ReturnStatement": {
      const { argument } = node;
      const statements = parseNode(argument);
      return $RETURN(statements);
    }
    case "MethodDefinition": {
      const { key, value } = node;

      return $FUNCTION(key, value.params, parseNode(value.body));
    }
    default: {
      throw new Error(`ParserError: Unknown node type '${node.type}'`);
    }
  }
}

module.exports.parse = parse;
