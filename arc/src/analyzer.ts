import * as Matrix from "./lib/Matrix";
import * as nucleoid from "./lib/nucleoid";
import * as llm from "./lib/llm";
import * as terminal from "./lib/terminal";
import * as responseFormat from "./lib/response_format";

export async function declarations({ train_dataset }) {
  terminal.step("Inferring declarations");

  const { declarations } = await llm.generate({
    messages: [
      {
        role: "user",
        content: JSON.stringify({
          operation: "infer_declarations",
          train_dataset,
        }),
      },
    ],
    response_format: responseFormat.create("analyzer_declarations", {
      declarations: {
        type: "array",
        minItems: 1,
        items: responseFormat.code,
      },
    }),
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
        content: JSON.stringify({
          operation: "segment_training_instances",
          declarations,
          input_matrix,
          output_matrix,
          query: { class: "Obj" },
        }),
      },
    ],
    response_format: responseFormat.create("analyzer_instances", {
      instances: {
        type: "array",
        items: responseFormat.object({
          input_instance_matrix: responseFormat.grid(
            input_matrix.length,
            input_matrix[0].length
          ),
          output_instance_matrix: responseFormat.grid(
            output_matrix.length,
            output_matrix[0].length
          ),
        }),
      },
    }),
  });

  instances.forEach((instance, index: number) => {
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
        content: JSON.stringify({
          operation: "construct_training_instance",
          instance_name,
          declarations,
          input_instance_matrix,
          output_instance_matrix,
        }),
      },
    ],
    response_format: responseFormat.create("analyzer_value", {
      implementation: responseFormat.code,
    }),
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
