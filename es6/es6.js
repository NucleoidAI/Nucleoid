const Token = require("../token");
const ES6$VARIABLE = require("./es6$variable");
const ES6$CLASS = require("./es6$class");
const ES6$ASSIGNMENT = require("./es6$assignment");
const ES6$BLOCK = require("./es6$block");
const ES6$DELETE = require("./es6$delete");
const ES6$IF = require("./es6$if");
const ES6$LET = require("./es6$let");
const $EXP = require("../$expression");
const ES6$THROW = require("./es6$throw");
const ES6$FUNCTION = require("./es6$function");
const ES6$FOR = require("./es6$for");

module.exports.compile = function compile(string) {
  let list = [];

  for (let offset = 0; offset < string.length; ) {
    let context = Token.next(string, offset);
    let check = Token.next(string, context.offset);

    if (check && check.token === "=") {
      check = Token.next(string, check.offset);

      if (check && check.token !== "=") {
        context = ES6$ASSIGNMENT(string, offset);
        if (context.statement) {
          list.push(context.statement);
        }

        offset = context.offset;
        continue;
      }
    }

    if (context.token === "var") context = ES6$VARIABLE(string, offset);
    else if (context.token === "if") context = ES6$IF(string, offset);
    else if (context.token === "class") context = ES6$CLASS(string, offset);
    else if (context.token === "{") context = ES6$BLOCK(string, offset);
    else if (context.token === "delete") context = ES6$DELETE(string, offset);
    else if (context.token === "let") context = ES6$LET(string, offset);
    else if (context.token === "new") context = ES6$VARIABLE(string, offset);
    else if (context.token === "throw") context = ES6$THROW(string, offset);
    else if (context.token === "function")
      context = ES6$FUNCTION(string, offset);
    else if (context.token === "for") context = ES6$FOR(string, offset);
    else context = $EXP(string, offset);

    if (context.statement) {
      list.push(context.statement);
    }

    offset = context.offset;
  }

  return list;
};
