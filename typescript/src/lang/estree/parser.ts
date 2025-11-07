import * as acorn from "acorn";

import $ASSIGNMENT from "../$nuc/$ASSIGNMENT";
import $BLOCK from "../$nuc/$BLOCK";
import $CLASS from "../$nuc/$CLASS";
import $DELETE from "../$nuc/$DELETE";
import $EXPRESSION from "../$nuc/$EXPRESSION";
import $FOR from "../$nuc/$FOR";
import $FUNCTION from "../$nuc/$FUNCTION";
import $IF from "../$nuc/$IF";
import $INSTANCE from "../$nuc/$INSTANCE";
import $RETURN from "../$nuc/$RETURN";
import $THROW from "../$nuc/$THROW";
import { Identifier } from "acorn";

interface Node {
  type: string;
  [key: string]: any;
}

interface EstreeNode {
  type: string;
  body: any[];
  expression?: any;
  start?: number;
  end?: number;
  [key: string]: any;
}

interface DeclarationNode extends Node {
  kind: string;
  declarations: {
    id: any;
    init: any;
  }[];
}

interface ClassNode extends Node {
  id: any;
  body: {
    body: any[];
  };
}

interface AssignmentNode extends Node {
  left: any;
  right: any;
}

interface BlockNode extends Node {
  body: any[];
}

interface IfNode extends Node {
  test: any;
  consequent: any;
  alternate: any;
}

interface ExpressionNode extends Node {
  expression: {
    type: string;
    operator?: string;
    argument?: any;
  };
}

interface NewNode extends Node {
  callee: any;
  arguments: any[];
}

interface ReturnNode extends Node {
  argument: any;
}

interface MethodNode extends Node {
  key: any;
  value: {
    params: any[];
    body: any;
  };
}

interface FunctionNode extends Node {
  id: any;
  params: any[];
  body: any;
}


interface ForOfNode extends Node {
  left: Identifier;
  right: any;
  body: {
    body: any[];
  };
}

interface ThrowNode extends Node {
  argument: any;
}

function parse(string: string, map: boolean = true) {
  const estree = acorn.parse(string, {
    ecmaVersion: 2020,
    allowReturnOutsideFunction: true,
  }) as EstreeNode;

  removeLocations(estree);

  return map
    ? estree.body.map(parseNode).filter((exp) => exp)
    : estree.body[0].expression;
}

function parseFunction(string: string) {
  const estree = acorn.parse(string, {
    ecmaVersion: 2020,
    allowReturnOutsideFunction: true,
  }) as EstreeNode;

  if (estree.body[0].type === "ExpressionStatement") {
    return estree.body[0].expression;
  } else {
    return estree.body[0];
  }
}

function parseNode(node: Node) {
  switch (node.type) {
    case "VariableDeclaration": {
      const {
        kind,
        declarations: [declaration],
      } = node as DeclarationNode;

      const { id, init } = declaration;

      return $ASSIGNMENT(
        kind.toUpperCase() as "VAR" | "LET" | "CONST",
        id,
        init
      );
    }
    case "ClassDeclaration": {
      const {
        id,
        body: { body },
      } = node as ClassNode;

      const methods = body.map((method) => parseNode(method));

      return $CLASS(id, methods);
    }
    case "AssignmentExpression": {
      const { left, right } = node as AssignmentNode;
      return $ASSIGNMENT(null as null, left, right);
    }
    case "BlockStatement": {
      const statements = (node as BlockNode).body.map(parseNode);
      return $BLOCK(statements);
    }
    case "IfStatement": {
      const { test, consequent, alternate } = node as IfNode;
      const parsedConsequent = parseNode(consequent);
      const parsedAlternate = alternate ? parseNode(alternate) : undefined;

      return $IF(test, parsedConsequent, parsedAlternate);
    }
    case "ExpressionStatement": {
      const { expression } = node as ExpressionNode;

      if (["AssignmentExpression", "NewExpression"].includes(expression.type)) {
        return parseNode(expression);
      } else if (
        expression.type === "UnaryExpression" &&
        expression.operator === "delete"
      ) {
        return $DELETE(expression.argument);
      } else {
        return $EXPRESSION(expression);
      }
    }
    case "NewExpression": {
      const { callee, arguments: args } = node as NewNode;
      return $INSTANCE(callee, undefined, undefined, args[0]);
    }
    case "ReturnStatement": {
      const { argument } = node as ReturnNode;
      const statements = parseNode(argument);
      return $RETURN(statements);
    }
    case "MethodDefinition": {
      const { key, value } = node as MethodNode;

      return $FUNCTION(key, value.params, parseNode(value.body));
    }
    case "FunctionDeclaration": {
      const { id, params, body } = node as FunctionNode;

      return $FUNCTION(id, params, parseNode(body));
    }
    case "ForOfStatement": {
      const { left, right, body } = node as ForOfNode;

      const bodyExpressions = body.body.map(parseNode);
      return $FOR(left as Identifier, right, bodyExpressions);
    }
    case "ThrowStatement": {
      const { argument } = node as ThrowNode;

      return $THROW(argument);
    }
    case "EmptyStatement": {
      return;
    }
    default: {
      return $EXPRESSION(node);
    }
  }
}

function removeLocations(estree: any): void {
  for (let key in estree) {
    delete estree.start;
    delete estree.end;

    if (estree[key] !== null && typeof estree[key] === "object") {
      removeLocations(estree[key]);
    } else if (Array.isArray(estree[key])) {
      estree[key].forEach((node: any) => removeLocations(node));
    }
  }
}

export { parse, parseFunction };
