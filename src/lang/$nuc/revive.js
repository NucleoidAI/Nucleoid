const $ = {
  $ALIAS: require("./$ALIAS"),
  $ASSIGNMENT: require("./$ASSIGNMENT"),
  $BLOCK: require("./$BLOCK"),
  $CALL: require("./$CALL"),
  $CLASS: require("./$CLASS"),
  $DELETE: require("./$DELETE"),
  $EXPRESSION: require("./$EXPRESSION"),
  $FOR: require("./$FOR"),
  $FUNCTION: require("./$FUNCTION"),
  $IF: require("./$IF"),
  $INSTANCE: require("./$INSTANCE"),
  $LET: require("./$LET"),
  $PROPERTY: require("./$PROPERTY"),
  $RETURN: require("./$RETURN"),
  $THROW: require("./$THROW"),
  $VARIABLE: require("./$VARIABLE"),
  Expression: require("../../Expression"),
  EXPRESSION: require("../../nuc/EXPRESSION"),
};

function revive(statement) {
  if (Array.isArray(statement)) {
    return statement.map((s) => revive(s));
  } else if (typeof statement === "object" && statement !== null) {
    let object;

    if (statement.iof) {
      object = new $[statement.iof]();
    } else {
      object = {};
    }

    for (const key in statement) {
      const property = statement[key];

      if (
        Array.isArray(property) ||
        (typeof object === "object" && object !== null)
      ) {
        object[key] = revive(property);
      } else {
        object[key] = property;
      }
    }

    return object;
  } else {
    return statement;
  }
}

module.exports = revive;
