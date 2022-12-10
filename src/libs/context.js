let nucleoid;

setImmediate(() => {
  nucleoid = require("../../index");
});

let _context = [];

function load(context) {
  _context = _context.concat(context);
}

function run(context = []) {
  load(context);

  _context.forEach(({ definition, options }) =>
    nucleoid.run(definition, options)
  );

  _context = [];
}

module.exports.load = load;
module.exports.run = run;
