const arc = require("./arc.md");
const nucleoid = require("./nucleoid.md");
const Zoom = require("../lib/Zoom");
const fs = require("fs");

const dataset = [
  require("./dataset.core.json"),
  require("./dataset.line.json"),
  require("./dataset.curve.json"),
].map(({ declarations, dataset }) => ({
  declarations,
  dataset: dataset.map(({ input_matrix, output_matrix, instances }) => ({
    input_matrix,
    output_matrix,
    instances: instances.map(
      ({
        instance_name,
        input_code,
        input_instance,
        output_instance,
        output_value,
      }) => ({
        instance_name,
        input_code,
        input_instance,
        input_object: Zoom.focus(input_instance),
        output_object: Zoom.focus(output_instance),
        output_value,
      })
    ),
  })),
}));

// fs.writeFileSync(
//   "dataset.jsonl",
//   JSON.stringify({
//     messages: dataset.map((d) => JSON.stringify(d)),
//   })
// );

module.exports.document = () => {
  return `
    ${nucleoid}
    instruct_dataset:
    ${JSON.stringify(dataset)}      
  `;
};
