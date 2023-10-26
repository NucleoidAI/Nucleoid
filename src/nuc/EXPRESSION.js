const state = require("../state");
const graph = require("../graph");
const Local = require("../lib/local");
const Id = require("../lib/identifier");
const Token = require("../lib/token");
const Node = require("./Node");
const LET = require("./LET");
const Evaluation = require("../lang/ast/Evaluation");

let Stack;
let $CALL;

setImmediate(() => {
  Stack = require("../stack");
  $CALL = require("../lang/$nuc/$CALL");
});

class EXPRESSION {
  constructor(tokens) {
    this.instanceof = this.constructor.name;
    this.tokens = tokens;
  }

  before() {}

  run() {
    const tokens = this.tokens;

    const expression = tokens.traverse((node) => {
      switch (node.type) {
        case "Literal": {
          return node.raw;
        }
        case "Identifier": {
          return node.name;
        }
        case "MemberExpression": {
          return `state.${node.object.name}.${node.property.name}`;
        }
        case "CallExpression": {
          return `${node.callee.object.name}.${node.callee.property.name}()`;
        }
        case "ArrayExpression": {
          return `[${node.elements.map((element) => element.raw).join(",")}]`;
        }
        default: {
          throw new Error(`ParserError: Unknown node type '${node.type}'`);
        }
      }
    });

    const transactions = tokens.filter((node) => {
      switch (node.type) {
        case "CallExpression": {
          return {
            exec: `${node.callee.object.name}.${node.callee.property.name}()`,
          };
        }
      }
    });

    return new Evaluation(expression.join(""), transactions);
  }

  next() {}

  graph() {}
}

module.exports = EXPRESSION;
