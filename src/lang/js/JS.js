const Token = require("../../lib/token");
const JS$VARIABLE = require("./JS$VARIABLE");
const JS$ASSIGNMENT = require("./JS$ASSIGNMENT");
const JS$BLOCK = require("./JS$BLOCK");
const JS$DELETE = require("./JS$DELETE");
const JS$IF = require("./JS$IF");
const $EXP = require("../$nuc/$EXPRESSION");
const JS$THROW = require("./JS$THROW");
const JS$FUNCTION = require("./JS$FUNCTION");
const JS$FOR = require("./JS$FOR");
const JS$RETURN = require("./JS$RETURN");
const ES6$CLASS = require("../es6/ES6$CLASS");
const ES6$LET = require("../es6/ES6$LET");

function compile(string, offset) {
  offset = offset || 0;
  let statements = [];

  while (offset < string.length) {
    let context = Token.next(string, offset);

    if (!context || !context.token) {
      offset++;
      continue;
    }

    let check = Token.next(string, context.offset);

    if (check?.token === "=") {
      check = Token.next(string, check.offset);

      if (check && check.token !== "=") {
        context = JS$ASSIGNMENT(string, offset);
        if (context.statement) {
          statements.push(context.statement);
        }

        offset = context.offset;
        continue;
      }
    } else if (check?.token === "[") {
      check = Token.nextBracket(string, check.offset);

      check = Token.next(string, check.offset);

      if (check && check.token === "=") {
        check = Token.next(string, check.offset);

        if (check && check.token !== "=") {
          context = JS$ASSIGNMENT(string, offset);
          if (context.statement) {
            statements.push(context.statement);
          }

          offset = context.offset;
          continue;
        }
      }
    }

    if (context.token === "var") context = JS$VARIABLE(string, offset);
    else if (context.token === "if") context = JS$IF(string, offset);
    else if (context.token === "class") context = ES6$CLASS(string, offset);
    else if (context.token === "{") {
      const check = Token.next(string, context.offset);
      let skip = false;
      if (check) {
        const colon = Token.next(string, check.offset);

        if (colon?.token === ":") {
          context = $EXP(string, offset);
          skip = true;
        }
      }

      if (!skip) {
        context = JS$BLOCK(string, offset);
      }
    } else if (context.token === "delete") context = JS$DELETE(string, offset);
    else if (context.token === "let") context = ES6$LET(string, offset, false);
    else if (context.token === "const") context = ES6$LET(string, offset, true);
    else if (context.token === "new") context = JS$VARIABLE(string, offset);
    else if (context.token === "throw") context = JS$THROW(string, offset);
    else if (context.token === "function")
      context = JS$FUNCTION(string, offset);
    else if (context.token === "for") context = JS$FOR(string, offset);
    else if (context.token === "return") context = JS$RETURN(string, offset);
    else context = $EXP(string, offset);

    if (context.statement) {
      statements.push(context.statement);
    }

    offset = context.offset;
  }

  return { statements, offset };
}

module.exports.compile = compile;
