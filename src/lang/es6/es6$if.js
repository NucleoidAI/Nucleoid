const Token = require("../../token");
const $EXP = require("../$/$expression");
const $BLOCK = require("../$/$block");
const $IF = require("../$/$if");
const ES6 = require("./es6");

module.exports = function ES6$IF(string, offset) {
  let context = Token.next(string, offset);

  if (context && context.token === "if")
    context = Token.next(string, context.offset);

  if (context && context.token === "(") {
    context = $EXP(string, context.offset);
    let condition = context.statement;

    context = Token.next(string, context.offset);
    context = Token.nextBlock(string, context.offset);
    let trueBlock = ES6.compile(context.block).statements;

    let elsePoint = Token.next(string, context.offset);
    let falseBlock;

    if (elsePoint && elsePoint.token === "else") {
      let check = Token.next(string, elsePoint.offset);

      if (check && check.token === "if") {
        context = ES6$IF(string, elsePoint.offset);
        falseBlock = context.statement;
      } else {
        context = Token.next(string, elsePoint.offset);
        context = Token.nextBlock(string, context.offset);
        const { statements } = ES6.compile(context.block);
        falseBlock = $BLOCK(statements);
      }
    }

    return {
      statement: $IF(condition, $BLOCK(trueBlock), falseBlock),
      offset: context.offset,
    };
  }
};
