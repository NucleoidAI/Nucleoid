import * as Matrix from "./lib/Matrix";
import * as nucleoid from "./lib/nucleoid";
import * as llm from "./lib/llm";

export async function declarations({ train_dataset }) {
  console.log("Analyzing declarations...");

  const { declarations } = await llm.generate({
    messages: [
      {
        role: "user",
        content: `
          train_dataset:
          ${JSON.stringify(train_dataset)}
          json_format:
          { "declarations": <NUC_DECLARATIONS> }
        `,
      },
    ],
  });

  console.debug("declarations:");
  console.debug(declarations);

  return { declarations };
}

export async function instances({ declarations, input_matrix, output_matrix }) {
  console.log("Extracting instances...");

  const { instances } = await llm.generate({
    messages: [
      {
        role: "user",
        content: `
          declarations:
          ${JSON.stringify(declarations)}
          input_matrix:
          ${JSON.stringify(input_matrix)}
          output_matrix:
          ${JSON.stringify(output_matrix)}
          query:
          Obj;
          json_format:
          {
            "instances": [{
              "input_instance_matrix": <INPUT_INSTANCE_MATRIX>,
              "output_instance_matrix": <OUTPUT_INSTANCE_MATRIX>
            }]
          }
        `,
      },
    ],
  });

  instances.forEach((instance) => {
    Matrix.toString(instance.input_instance_matrix);
    console.debug("");
    Matrix.toString(instance.output_instance_matrix);
    console.debug("--");
  });
  console.log("");

  return { instances };
}

export async function value({
  instance_name,
  declarations,
  input_instance_matrix,
  output_instance_matrix,
}) {
  console.log("Calculating value...");

  const { input_code } = await llm.generate({
    messages: [
      {
        role: "user",
        content: `
          instance_name:
          ${instance_name}
          declarations:
          ${declarations.join("\n")}
          input_instance_matrix:
          ${JSON.stringify(input_instance_matrix)}
          output_instance_matrix:
          ${JSON.stringify(output_instance_matrix)}
          json_format:
          { "input_code": <INPUT_CODE> }
        `,
      },
    ],
  });

  console.log("Creating instance in Nucleoid...");
  const input_value = await nucleoid.run(
    [declarations[0], input_code].join("\n\n")
  );
  const output_value = await nucleoid.run(
    [...declarations, input_code].join("\n\n")
  );

  console.debug("input_code:");
  console.debug(input_code);
  console.debug("input_value:");
  console.debug(input_value);
  console.debug("output_value:");
  console.debug(output_value);

  return { input_code, input_value, output_value };
}
