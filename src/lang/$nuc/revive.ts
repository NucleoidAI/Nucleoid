import $ from "../$nuc/$";
import $ALIAS from "./$ALIAS";
import $ASSIGNMENT from "./$ASSIGNMENT";
import $BLOCK from "./$BLOCK";
import $CALL from "./$CALL";
import $CLASS from "./$CLASS";
import $DELETE from "./$DELETE";
import $EXPRESSION from "./$EXPRESSION";
import $FOR from "./$FOR";
import $FUNCTION from "./$FUNCTION";
import $IF from "./$IF";
import $INSTANCE from "./$INSTANCE";
import $LET from "./$LET";
import $PROPERTY from "./$PROPERTY";
import $RETURN from "./$RETURN";
import $THROW from "./$THROW";
import $VARIABLE from "./$VARIABLE";
import EXPRESSION from "../../nuc/EXPRESSION";
import Expression from "../../Expression";

const _$ = {
  $ALIAS,
  $ASSIGNMENT,
  $BLOCK,
  $CALL,
  $CLASS,
  $DELETE,
  $EXPRESSION,
  $FOR,
  $FUNCTION,
  $IF,
  $INSTANCE,
  $LET,
  $PROPERTY,
  $RETURN,
  $THROW,
  $VARIABLE,
  EXPRESSION,
  Expression,
};

function revive(statements: $[]): $[] {
  return statements.map((statement) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let object: any;

    if (statement.iof) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      object = new (_$[statement.iof as keyof typeof _$] as any)();
    } else {
      object = {};
    }

    for (const key in statement) {
      const property = statement[key];

      if (
        Array.isArray(property) ||
        (typeof property === "object" && property !== null)
      ) {
        object[key] = revive(property);
      } else {
        object[key] = property;
      }
    }

    return object;
  });
}

export default revive;
