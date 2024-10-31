const nucleoid = require("./nucleoid");
const Matrix = require("../lib/Matrix");
const Zoom = require("./Zoom");
const llm = require("./llm");

async function instances({ train_dataset, test_input_matrix }) {
  console.log("Analyzing test_input_matrix...");
  console.log("");

  console.debug("test_input_matrix:");
  Matrix.toString(test_input_matrix);
  console.debug("");

  const rows = test_input_matrix.length;
  const cols = test_input_matrix[0].length;

  const { instances } = await llm.generate({
    messages: [
      {
        role: "user",
        content: `
          ${JSON.stringify({
            class: [train_dataset.declarations[0]],
            dataset: train_dataset.dataset.map(
              ({ input_matrix, instances }) => ({
                input_matrix,
                instances: instances.map(
                  ({ instance_name, input_object, input_code }) => ({
                    instance_name,
                    input_code,
                    input_object,
                  })
                ),
              })
            ),
          })}
          input_matrix:
          ${JSON.stringify(test_input_matrix)}
          json_format:
          {
            "instances": [{
              "input_object": { "x_position": <INPUT_X_POSITION>, "y_position": <INPUT_Y_POSITION>, "object_matrix": <INPUT_OBJECT_MATRIX> }
            }]
          }
        `,
      },
    ],
  });

  console.debug(`${instances.length} instances`);
  instances.forEach((instance) => {
    const input_instance = Zoom.enlarge(instance.input_object, rows, cols);
    instance.input_instance = input_instance;
    Matrix.toString(input_instance);
    console.debug("--");
  });

  return { instances };
}

async function value({
  test_session_id,
  instance_name,
  train_dataset,
  input_object,
}) {
  console.log("Calculating test value...");
  const { input_code } = await llm.generate({
    messages: [
      {
        role: "user",
        content: `
          instance_name:
          ${instance_name}
          train_dataset:
          ${JSON.stringify({
            declarations: train_dataset.declarations,
            dateset: train_dataset.dataset.map(({ instances }) => ({
              instances: instances.map(
                ({ instance_name, input_object, input_code }) => ({
                  instance_name,
                  input_object,
                  input_code,
                })
              ),
            })),
          })}
          input_object:
          ${JSON.stringify(input_object)}
          json_format:
          { "input_code": <NUCLEOID_CODE> }
        `,
      },
    ],
  });

  console.log("Creating test instance in Nucleoid...");
  const output_value = await nucleoid.run(test_session_id, input_code);

  console.debug("input_code:");
  console.debug(input_code);
  console.debug("output_value:");
  console.debug(output_value);

  return { output_value };
}

async function output_instance({
  test_input_matrix,
  result_matrix,
  train_dataset,
  input_object,
  output_value,
}) {
  console.log("Extracting test output_instance...");
  const { output_object } = await llm.generate({
    messages: [
      {
        role: "user",
        content: `
          train_dataset:
          ${JSON.stringify(train_dataset)}
          input_matrix:
          ${JSON.stringify(test_input_matrix)}
          output_matrix:
          ${JSON.stringify(result_matrix)}
          input_object:
          ${JSON.stringify(input_object)}
          output_value:
          ${JSON.stringify(output_value)}
          json_format:
          {
            "output_object": { "x_position": <OUTPUT_X_POSITION>, "y_position": <OUTPUT_Y_POSITION>, "object_matrix": <OUTPUT_OBJECT_MATRIX> }
          }
        `,
      },
    ],
  });

  const rows = result_matrix.length;
  const cols = result_matrix[0].length;
  const output_instance = Zoom.enlarge(output_object, rows, cols);

  console.debug("output_instance:");
  Matrix.toString(output_instance);
  console.debug("output_object:");
  Matrix.toString(output_object.object_matrix);

  return { output_instance };
}

module.exports = { instances, value, output_instance };
