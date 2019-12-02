var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Local = require("./local");
var Value = require("./value");
var Identifier = require("./identifier");
var Node = require("./node");
var Token = require("./token");

module.exports = class EXPRESSION extends Value {
  constructor(tokens) {
    super();
    this.tokens = tokens;
  }

  prepare(scope, self) {
    this.tokens = this.tokens
      .map(token => (token === self ? token + ".value" : token))
      .map(token => {
        let parts = Identifier.splitLast(token);
        if (parts[0] && parts[1] && parts[0] === "value") {
          if (Local.check(scope, parts[1])) {
            return parts[1];
          }

          if (graph[parts[1]] !== undefined) {
            let value = eval("state." + parts[1]);
            return JSON.stringify(value);
          } else throw TypeError();
        } else return token;
      });
  }

  run(scope) {
    try {
      let tokens = this.tokens
        .map(token => (token = Local.reference(scope, token)))
        .map(token => {
          let parts = token.split(".");

          try {
            if (Local.check(scope, parts[0])) {
              let reference = Local.retrieve(scope, token);
              let value = eval(reference);

              if (value === undefined) throw 0;
              return reference;
            } else if (graph[parts[0]]) {
              let reference = "state." + Identifier.reference(token);
              let value = eval(reference);

              if (value === undefined) throw 0;
              return reference;
            } else {
              return token;
            }
          } catch (error) {
            if (error instanceof TypeError) throw 1;
            else if (error === 0) throw 1;
            else throw error;
          }
        });

      return eval(tokens.construct());
    } catch (error) {
      if (error instanceof Error) throw error;
      return undefined;
    }
  }

  next() {
    let list = [];

    for (let token of this.tokens) {
      if (token instanceof Token.FUNCTION) {
        let parts = Identifier.splitLast(token.string);
        if (parts[1] && graph[parts[1]]) {
          for (let node in graph[parts[1]].next)
            list.push(graph[parts[1]].next[node]);
        }
      }
    }

    return list;
  }

  graph(scope) {
    return this.tokens
      .list()
      .map(token => Local.reference(scope, token))
      .map(token => Identifier.reference(token))
      .filter(token => {
        if (graph[token]) return true;
        else if (graph[token.split(".")[0]]) {
          graph[token] = new Node();
          return true;
        }
      })
      .map(token => {
        let parts = Identifier.splitLast(token);
        if (parts[0] && parts[1] && parts[0] === "length") return parts[1];
        else return token;
      });
  }
};
