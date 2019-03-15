var graph = require("./graph");
var Token = require("./token");
var $VAR = require("./variable");
var $ASSIGN = require("./assignment");
var $IF = require("./if");
var $CLASS = require("./class");
var $EXP = require("./expression");

module.exports.compile = function(string) {
  let list = [];

  for (let offset = 0; offset < string.length; ) {
    let context = Token.next(string, offset);
    let check = Token.next(string, context.offset);

    if (check && check.token == "=") {
      if (graph.node[context.token]) {
        context = $ASSIGN(string, offset);
      } else {
        context = $VAR(string, offset);
      }
    } else {
      if (context.token == "var") {
        context = $VAR(string, offset);
      } else if (context.token == "if") {
        context = $IF(string, offset);
      } else if (context.token == "class") {
        context = $CLASS(string, offset);
      } else {
        context = $EXP(string, offset);
      }
    }

    offset = context.offset;
    list.push(context.statement);
  }

  return list;
};
