const acorn = require("acorn");
const $VARIABLE = require("../$nuc/$VARIABLE");
const $EXPRESSION = require("../$nuc/$EXPRESSION");
const $ASSIGNMENT = require("../$nuc/$ASSIGNMENT");
const $CLASS = require("../$nuc/$CLASS");
const $INSTANCE = require("../$nuc/$INSTANCE");
const $BLOCK = require("../$nuc/$BLOCK");
const $IF = require("../$nuc/$IF");
const $RETURN = require("../$nuc/$RETURN");
const $LET = require("../$nuc/$LET");
const Expression = require("../ast/Expression");
const Identifier = require("../ast/Identifier");

function parse(string, map = true) {
  const estree = acorn.parse(string, { ecmaVersion: 2020 });

  return map
    ? estree.body[0].expression.callee.body.body.map(parseNode)
    : estree.body[0].expression;
}

function parseNode(node) {
  switch (node.type) {
    case "MemberExpression":
    case "Identifier": {
      return $EXPRESSION(new Expression(node));
    }
    case "VariableDeclaration": {
      const {
        kind,
        declarations: [declaration],
      } = node;
      const { id, init } = declaration;

      if (init.type === "NewExpression") {
        const cls = new Identifier(init.callee);
        const name = new Identifier(id);
        return $INSTANCE(cls, null, name, init.arguments);
      } else {
        if (kind === "let") {
          return $LET(
            new Identifier(id),
            $EXPRESSION(new Expression(init)),
            false
          );
        } else if (kind === "const") {
          return $LET(
            new Identifier(id),
            $EXPRESSION(new Expression(init)),
            true
          );
        } else if (kind === "var") {
          return $VARIABLE(
            new Identifier(id),
            $EXPRESSION(new Expression(init))
          );
        } else {
          throw new Error("Unknown variable type");
        }
      }
    }
    case "ClassDeclaration": {
      const {
        id,
        body: { body },
      } = node;

      const name = new Identifier(id);
      return $CLASS(name, body);
    }
    case "AssignmentExpression": {
      const { left, right } = node;
      const name = new Identifier(left);

      if (node.right.type === "NewExpression") {
        const cls = new Identifier(node.right.callee);

        if (left.type === "MemberExpression") {
          const object = new Identifier(left.object);
          const name = new Identifier(left.property);
          return $INSTANCE(cls, object, name, node.right.arguments);
        } else if (left.type === "Identifier") {
          return $INSTANCE(cls, null, name, node.right.arguments);
        } else {
          throw new Error("Unknown identifier type");
        }
      } else {
        return $ASSIGNMENT(name, $EXPRESSION(new Expression(right)));
      }
    }
    case "BlockStatement": {
      const statements = node.body.map(parseNode);
      return $BLOCK(statements);
    }
    case "IfStatement": {
      const condition = $EXPRESSION(new Expression(node.test));
      const consequent = parseNode(node.consequent);
      const alternate = node.alternate ? parseNode(node.alternate) : null;

      return $IF(condition, consequent, alternate);
    }
    case "ExpressionStatement": {
      const { expression } = node;

      if (expression.type === "AssignmentExpression") {
        return parseNode(expression);
      } else {
        return $EXPRESSION(new Expression(expression));
      }
    }
    case "ReturnStatement": {
      const { argument } = node;
      const statements = parseNode(argument);
      return $RETURN(statements);
    }
    default: {
      throw new Error(`ParserError: Unknown node type '${node.type}'`);
    }
  }
}

module.exports.parse = parse;
