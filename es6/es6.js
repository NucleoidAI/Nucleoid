var Token = require("../token");
var ES6$VARIABLE = require("./es6$variable");
var ES6$CLASS = require("./es6$class");
var ES6$ASSIGNMENT = require("./es6$assignment");
var ES6$BLOCK = require("./es6$block");
var ES6$DELETE = require("./es6$delete");
var ES6$IF = require("./es6$if");
var ES6$LET = require("./es6$let");
var $VALUE = require("../$value");

module.exports.compile = function compile(string) {
  let list = [];

  for (let offset = 0; offset < string.length; ) {
    let context = Token.next(string, offset);
    let check = Token.next(string, context.offset);

    if (check && check.token === "=") context = ES6$ASSIGNMENT(string, offset);
    else {
      if (context.token === "var") context = ES6$VARIABLE(string, offset);
      else if (context.token === "if") context = ES6$IF(string, offset);
      else if (context.token === "class") context = ES6$CLASS(string, offset);
      else if (context.token === "{") context = ES6$BLOCK(string, offset);
      else if (context.token === "delete") context = ES6$DELETE(string, offset);
      else if (context.token === "let") context = ES6$LET(string, offset);
      else context = $VALUE(string, offset);
    }

    list.push(context.statement);
    offset = context.offset;
  }

  return list;
};
