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
          { "declarations": <DECLARATIONS> }
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

  const { implementation } = await llm.generate({
    messages: [
      {
        role: "user",
        content: `
          instance_name:
          ${instance_name}
          declarations:
          ${JSON.stringify(declarations)}
          input_instance_matrix:
          ${JSON.stringify(input_instance_matrix)}
          output_instance_matrix:
          ${JSON.stringify(output_instance_matrix)}
          json_format:
          { "implementation": <IMPLEMENTATION> }
        `,
      },
    ],
  });

  console.log("Creating instance in Nucleoid...");
  const input_value = await nucleoid.run(
    [declarations[0], implementation].join("\n\n")
  );
  const output_value = await nucleoid.run(
    [...declarations, implementation].join("\n\n")
  );

  console.debug("implementation:");
  console.debug(implementation);
  console.debug("input_value:");
  console.debug(input_value);
  console.debug("output_value:");
  console.debug(output_value);

  return { implementation, input_value, output_value };
}
