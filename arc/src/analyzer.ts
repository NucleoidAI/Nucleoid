import * as Matrix from "./lib/Matrix";
import * as nucleoid from "./lib/nucleoid";
import * as llm from "./lib/llm";
import * as terminal from "./lib/terminal";

export async function declarations({ train_dataset }) {
  terminal.step("Inferring declarations");

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

  terminal.code("Declarations", declarations);

  return { declarations };
}

export async function instances({ declarations, input_matrix, output_matrix }) {
  terminal.step("Segmenting training objects");

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

  instances.forEach((instance, index) => {
    terminal.grid(
      `Object ${index + 1} · Input`,
      instance.input_instance_matrix
    );
    terminal.grid(
      `Object ${index + 1} · Output`,
      instance.output_instance_matrix
    );
  });

  return { instances };
}

export async function value({
  instance_name,
  declarations,
  input_instance_matrix,
  output_instance_matrix,
}) {
  terminal.step(`Constructing ${instance_name}`);

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

  terminal.step("Evaluating declarative rules");
  const input_value = await nucleoid.run(
    [declarations[0], implementation].join("\n\n")
  );
  const output_value = await nucleoid.run(
    [...declarations, implementation].join("\n\n")
  );

  terminal.grid("Input crop", Matrix.crop(input_instance_matrix));
  terminal.grid("Output crop", Matrix.crop(output_instance_matrix));
  terminal.code(`Implementation · ${instance_name}`, implementation);
  terminal.value("Input state", input_value);
  terminal.value("Output state", output_value);

  return { implementation, input_value, output_value };
}
