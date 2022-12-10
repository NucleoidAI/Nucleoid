const Token = require("../../utils/token");
const JS$VARIABLE = require("./js$variable");
const JS$ASSIGNMENT = require("./js$assignment");
const JS$BLOCK = require("./js$block");
const JS$DELETE = require("./js$delete");
const JS$IF = require("./js$if");
const $EXP = require("../$nuc/$expression");
const JS$THROW = require("./js$throw");
const JS$FUNCTION = require("./js$function");
const JS$FOR = require("./js$for");
const JS$RETURN = require("./js$return");
const ES6$CLASS = require("../es6/es6$class");
const ES6$LET = require("../es6/es6$let");

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
