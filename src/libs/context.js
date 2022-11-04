let nucleoid;

setImmediate(() => {
  nucleoid = require("../../index");
});

let _context = [];

const load = (context) => {
  _context = _context.concat(context);
};

const run = (context = []) => {
  load(context);

  _context.forEach(({ definition, options }) =>
    nucleoid.run(definition, options)
  );

  _context = [];
};

module.exports = { load, run };
