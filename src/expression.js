const state = require("./state");
const graph = require("./graph");
const Local = require("./local");
const Identifier = require("./identifier");
const Node = require("./node");
const Token = require("./token");
const argv = require("yargs").argv;
let Stack;
let $CALL;

setImmediate(() => (Stack = require("./stack")));
setImmediate(() => ($CALL = require("./lang/$/$call")));

class EXPRESSION {
  constructor(tokens) {
    this.tokens = tokens;
  }

  before(scope, self) {
    this.tokens = this.tokens
      .map((token) => (token === self ? token + ".value" : token))
      .map((token) => {
        let parts = Identifier.splitLast(token);
        if (parts[0] && parts[1] && parts[0] === "value") {
          if (Local.check(scope, parts[1])) {
            return parts[1];
          }

          try {
            let value = state.run(scope, "state." + parts[1]);

            if (value === undefined)
              throw ReferenceError(`${parts[1]} is not defined`);

            return JSON.stringify(value);
          } catch (error) {
            throw ReferenceError(`${parts[1]} is not defined`);
          }
        } else return token;
      });
  }

  run(scope, skip) {
    try {
      if (this.tokens[0].string === "{") {
        return this.tokens.construct();
      }

      for (let i = 0; i < this.tokens.length; i++) {
        const token = this.tokens[i];

        if (token instanceof Token.CALL && graph[token.string]) {
          const value = Stack.process([$CALL(token.string, token.params)]);
          this.tokens[i] = new Token(
            value === undefined
              ? "undefined"
              : value === null
              ? "null"
              : value.toString()
          );
        }
      }

      let tokens = this.tokens
        .map((token) => (token = Local.reference(scope, token)))
        .map((token) => {
          let parts = token.split(".");

          try {
            if (Local.check(scope, parts[0])) {
              return Local.retrieve(scope, token);
            } else if (graph[parts[0]]) {
              let reference = "state." + Identifier.reference(token);
              let value = state.run(scope, reference);

              if (value === undefined && !skip) throw 0;
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

      if (argv.log === true || argv.l === true) console.log(tokens.construct());
      return tokens.construct();
    } catch (error) {
      if (error instanceof Error) throw error;
      return "undefined";
    }
  }

  next() {
    let list = [];

    for (let token of this.tokens) {
      if (token instanceof Token.CALL) {
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
      .map((token) => Local.reference(scope, token))
      .map((token) => Identifier.reference(token))
      .filter((token) => {
        if (graph[token]) return true;
        else if (graph[token.split(".")[0]]) {
          graph[token] = new Node();
          return true;
        }
      })
      .map((token) => {
        let parts = Identifier.splitLast(token);
        if (parts[0] && parts[1] && parts[0] === "length") return parts[1];
        else return token;
      })
      .map((token) => {
        let fn;

        try {
          fn = state.run(scope, "state." + token);
        } catch (error) {
          return token;
        }

        if (typeof fn === "function") {
          let parts = Identifier.splitLast(token);
          return parts[1];
        } else {
          return token;
        }
      });
  }
}

EXPRESSION.prototype.instanceof = "EXPRESSION";
module.exports = EXPRESSION;
