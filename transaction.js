const state = require("./state").state; // eslint-disable-line no-unused-vars

let list = [];

module.exports.start = function () {
  list = [];
};

module.exports.end = function () {
  const result = list;
  list = [];
  return result;
};

module.exports.register = function (p1, p2, p3) {
  if (!p1) {
    return;
  }

  if (typeof p1 === "string") {
    let variable = p1;
    let expression = p2;
    let scope = p3; // eslint-disable-line no-unused-vars

    let exec;

    if (typeof expression === "string") {
      exec = `state.${variable}=${expression}`;
    } else {
      exec = `state.${variable}=expression`;
    }

    // eslint-disable-next-line no-eval
    list.push({ variable, value: eval(`state.${variable}`), exec });
    // eslint-disable-next-line no-eval
    eval(exec);
  } else {
    let object = p1;
    let property = p2;
    let value = p3;

    list.push({ object, property, value: object[property] });
    object[property] = value;
  }
};

module.exports.rollback = function () {
  while (list.length) {
    let transaction = list.pop();
    let variable = transaction.variable;
    let object = transaction.object;
    let property = transaction.property;
    let value = transaction.value;

    if (variable !== undefined) {
      // eslint-disable-next-line no-eval
      eval(`state.${variable}=value`);
    } else {
      if (value === undefined) {
        delete object[property];
      } else {
        object[property] = value;
      }
    }
  }
};
