import { ArrayExpression, Expression, Node } from "acorn";
import $Identifier from "./$Identifier";
import $Array from "./$Array";
import $New from "./$New";
import $Object from "./$Object";
import $Function from "./$Function";
import $Template from "./$Template";
import $Operator from "./$Operator";
import $Node from "./$Node";

function convert(node: Node | Expression): $Node {
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  const $Literal = require("./$Literal").default;

  switch (node.type) {
    case "Literal": {
      return new $Literal(node);
    }
    case "Identifier":
    case "MemberExpression": {
      return new $Identifier(node);
    }
    case "ArrayExpression": {
      const elements = (
        (node as ArrayExpression).elements as Array<Expression>
      ).map((el: Expression) => convert(el));

      return new $Array(elements);
    }
    case "NewExpression": {
      return new $New(node);
    }
    case "ObjectExpression": {
      return new $Object(node);
    }
    case "FunctionExpression":
    case "ArrowFunctionExpression": {
      return new $Function(node);
    }
    // case "CallExpression": {
    //   return new $Call(node);
    // }
    case "TemplateLiteral": {
      return new $Template(node);
    }
    default: {
      return new $Operator(node);
    }
  }
}

export default { convert };
