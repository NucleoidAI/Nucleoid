const Token = require("../../lib/token");
const $EXP = require("../$nuc/$expression");
const $BLOCK = require("../$nuc/$block");
const $IF = require("../$nuc/$if");
const JS = require("./js");

function JS$IF(string, offset) {
  let context = Token.next(string, offset);

  if (context && context.token === "if")
    context = Token.next(string, context.offset);

  if (context && context.token === "(") {
    context = $EXP(string, context.offset);
    let condition = context.statement;

    context = Token.next(string, context.offset);
    context = Token.nextBlock(string, context.offset);
    let trueBlock = JS.compile(context.block).statements;

    let elsePoint = Token.next(string, context.offset);
    let falseBlock;

    if (elsePoint && elsePoint.token === "else") {
      let check = Token.next(string, elsePoint.offset);

      if (check && check.token === "if") {
        context = JS$IF(string, elsePoint.offset);
        falseBlock = context.statement;
      } else {
        context = Token.next(string, elsePoint.offset);
        context = Token.nextBlock(string, context.offset);
        const { statements } = JS.compile(context.block);
        falseBlock = $BLOCK(statements);
      }
    }

    return {
      statement: $IF(condition, $BLOCK(trueBlock), falseBlock),
      offset: context.offset,
    };
  }
}

module.exports = JS$IF;
