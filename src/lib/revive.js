/* eslint-disable no-unused-vars */
/* eslint-disable no-eval */
let ALIAS,
  BLOCK$CLASS,
  BLOCK$INSTANCE,
  BLOCK,
  CLASS,
  DELETE$OBJECT,
  DELETE$VARIABLE,
  DELETE,
  EXPRESSION,
  FOR,
  IF$CLASS,
  IF$INSTANCE,
  IF,
  LET$CLASS,
  LET$OBJECT,
  LET,
  OBJECT$CLASS,
  OBJECT$INSTANCE,
  OBJECT,
  PROPERTY$CLASS,
  PROPERTY$INSTANCE,
  PROPERTY$REFERENCE,
  PROPERTY,
  REFERENCE,
  RETURN,
  THROW,
  VARIABLE;

setImmediate(() => {
  ALIAS = require("../nuc/ALIAS");
  BLOCK$CLASS = require("../nuc/BLOCK$CLASS");
  BLOCK$INSTANCE = require("../nuc/BLOCK$INSTANCE");
  BLOCK = require("../nuc/BLOCK");
  CLASS = require("../nuc/CLASS");
  DELETE$OBJECT = require("../nuc/DELETE$OBJECT");
  DELETE$VARIABLE = require("../nuc/DELETE$VARIABLE");
  DELETE = require("../nuc/DELETE");
  EXPRESSION = require("../nuc/EXPRESSION");
  FOR = require("../nuc/FOR");
  // const FUNCTION = require("../nuc/FUNCTION");
  IF$CLASS = require("../nuc/IF$CLASS");
  IF$INSTANCE = require("../nuc/IF$INSTANCE");
  IF = require("../nuc/IF");
  LET$CLASS = require("../nuc/LET$CLASS");
  LET$OBJECT = require("../nuc/LET$OBJECT");
  LET = require("../nuc/LET");
  OBJECT$CLASS = require("../nuc/OBJECT$CLASS");
  OBJECT$INSTANCE = require("../nuc/OBJECT$INSTANCE");
  OBJECT = require("../nuc/OBJECT");
  PROPERTY$CLASS = require("../nuc/PROPERTY$CLASS");
  PROPERTY$INSTANCE = require("../nuc/PROPERTY$INSTANCE");
  PROPERTY$REFERENCE = require("../nuc/PROPERTY$REFERENCE");
  PROPERTY = require("../nuc/PROPERTY");
  REFERENCE = require("../nuc/REFERENCE");
  RETURN = require("../nuc/RETURN");
  THROW = require("../nuc/THROW");
  VARIABLE = require("../nuc/VARIABLE");
});

let state, graph;

setImmediate(() => {
  state = require("../state").state;
  graph = require("../graph");
});

function revive(object, acc = {}) {
  const Token = require("../lib/token");
  const { ARRAY, CALL, FUNCTION } = require("../lib/token");

  if (!(object instanceof Object) && !Array.isArray(object)) {
    return;
  }

  if (Array.isArray(object)) {
    object.forEach((value) => {
      revive(value, acc);
    });
  } else {
    // Set the root object as the parent
    if (!acc.parent) {
      acc.parent = object;
    }

    if (object.id === undefined && object.$ref === undefined) {
      return;
    }

    if (acc[object.id] !== undefined) {
      return;
    } else {
      acc[object.id] = object;
    }

    if (object.constructor.name === "Object" && object.instanceof) {
      eval(`Object.setPrototypeOf(object, ${object.instanceof}.prototype)`);
    }

    Object.entries(object).forEach(([key, value]) => {
      if (key === "$ref") {
        acc.parent[acc.current] = eval(`${value.source}['${value.id}']`);
        return;
      }

      acc.parent = object;
      acc.current = key;
      revive(value, acc);
    });
  }
}

module.exports = revive;
