const REFERENCE = require("./reference");

let list = [];
let state; // eslint-disable-line no-unused-vars

setImmediate(() => (state = require("./state").state));

module.exports.start = function () {
  list = [];
};

module.exports.end = function () {
  const result = list;
  list = [];
  return result;
};

module.exports.register = function (p1, p2, p3, adjust) {
  if (!p1) return;

  if (typeof p1 === "string") {
    let variable = p1;
    let expression = p2;
    let scope = p3; // eslint-disable-line no-unused-vars

    if (expression instanceof REFERENCE) {
      let exec;
      exec = `state.${variable}=${expression.run()}`;

      // eslint-disable-next-line no-eval
      list.push({ variable, before: eval(`state.${variable}`) });
      // eslint-disable-next-line no-eval
      return eval(exec);
    } else {
      // eslint-disable-next-line no-eval
      let transaction = { variable, before: eval(`state.${variable}`) };

      let value;

      if (typeof expression === "string") {
        // eslint-disable-next-line no-eval
        value = eval(`state.${variable}=${expression}`);
      } else {
        // eslint-disable-next-line no-eval
        value = eval(`state.${variable}=expression`);
      }

      if (adjust)
        transaction.exec = `state.${variable}=${JSON.stringify(value)}`;

      state[variable] = value;
      list.push(transaction);
      return value;
    }
  } else {
    let object = p1;
    let property = p2;
    let value = p3;

    list.push({ object, property, before: object[property] });
    object[property] = value;
  }
};

module.exports.rollback = function () {
  while (list.length) {
    let transaction = list.pop();
    let variable = transaction.variable;
    let object = transaction.object;
    let property = transaction.property;
    let before = transaction.before;

    if (variable !== undefined) {
      // eslint-disable-next-line no-eval
      eval(`state.${variable}=before`);
    } else {
      if (before === undefined) {
        delete object[property];
      } else {
        object[property] = before;
      }
    }
  }
};
