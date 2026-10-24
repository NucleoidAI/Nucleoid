import * as nucleoid from "./lib/nucleoid";
import * as Matrix from "./lib/Matrix";
import * as llm from "./lib/llm";
import * as terminal from "./lib/terminal";
import * as responseFormat from "./lib/response_format";

export async function instances({
  train_dataset,
  declarations,
  test_input_matrix,
}: {
  train_dataset: {
    dataset: {
      input_matrix: number[][];
      output_matrix: number[][];
      instances: {
        instance_name: string;
        input_instance_matrix: number[][];
        output_instance_matrix: number[][];
        implementation: string;
        input_value: object;
        output_value: object;
      }[];
    }[];
    declarations: string[];
  };
  declarations: string[];
  test_input_matrix: number[][];
}) {
  terminal.step("Segmenting the test input");
  terminal.grid("Test input", test_input_matrix);

  const { instances } = await llm.generate<{
    instances: { input_instance_matrix: number[][] }[];
  }>({
    messages: [
      {
        role: "assistant",
        content: JSON.stringify({
          declarations,
          train_dataset: train_dataset.dataset.map(
            ({ input_matrix, output_matrix, instances }) => ({
              input_matrix,
              output_matrix,
              instances: instances.map(
                ({ input_instance_matrix, output_instance_matrix }) => ({
                  input_instance_matrix,
                  output_instance_matrix,
                })
              ),
            })
          ),
        }),
      },
      {
        role: "user",
        content: JSON.stringify({
          input_matrix: test_input_matrix,
        }),
      },
    ],
    response_format: responseFormat.create("visualizer_instances", {
      instances: {
        type: "array",
        items: responseFormat.object({
          input_instance_matrix: responseFormat.grid(
            test_input_matrix.length,
            test_input_matrix[0].length
          ),
        }),
      },
    }),
  });

  terminal.step(`${instances.length} objects identified`);
  instances.forEach(({ input_instance_matrix }, index: number) => {
    terminal.grid(`Test object ${index + 1}`, input_instance_matrix);
  });

  return { instances };
}

export async function value({
  declarations,
  train_dataset,
  instance_name,
  input_instances,
  input_instance_matrix,
}: {
  declarations: string[];
  train_dataset: {
    dataset: {
      input_matrix: number[][];
      output_matrix: number[][];
      instances: {
        instance_name: string;
        input_instance_matrix: number[][];
        output_instance_matrix: number[][];
        implementation: string;
        input_value: object;
        output_value: object;
      }[];
    }[];
    declarations: string[];
  };
  instance_name: string;
  input_instances: {
    input_value: object;
    input_instance_matrix: number[][];
  }[];
  input_instance_matrix: number[][];
}) {
  const { x, y } = Matrix.locate(input_instance_matrix);

  terminal.step(`Synthesizing ${instance_name}`);
  const { implementation } = await llm.generate<{
    orientation: number;
    implementation: string;
  }>({
    messages: [
      {
        role: "assistant",
        content: JSON.stringify({
          declarations,
          train_dataset: train_dataset.dataset.map(({ instances }) => ({
            instances: instances.map(
              ({ implementation, input_value, input_instance_matrix }) => ({
                implementation,
                input_value,
                input_instance_matrix: Matrix.crop(input_instance_matrix),
              })
            ),
          })),
          input_instances: input_instances.map(
            ({ input_value, input_instance_matrix }) => ({
              input_value,
              input_instance_matrix: Matrix.crop(input_instance_matrix),
            })
          ),
        }),
      },
      {
        role: "user",
        content: JSON.stringify({
          instance_name,
          input_instance_matrix: Matrix.crop(input_instance_matrix),
          input_x: x,
          input_y: y,
        }),
      },
    ],
    response_format: responseFormat.create("visualizer_value", {
      orientation: { type: "number" },
      implementation: responseFormat.code,
    }),
  });

  terminal.code(`Implementation · ${instance_name}`, implementation);

  terminal.step("Executing the object transformation");
  const input_value = nucleoid.snapshot(
    await nucleoid.run(
      [train_dataset.declarations[0], implementation].join("\n\n")
    )
  );
  const output_value = nucleoid.snapshot(
    await nucleoid.run(
      [...train_dataset.declarations, implementation].join("\n\n")
    )
  );

  terminal.value("Input state", input_value);
  terminal.value("Output state", output_value);

  return { input_value, output_value };
}

export async function output_instance_matrix({
  test_input_matrix,
  train_dataset,
  input_instances,
  input_instance_matrix,
  input_value,
  output_value,
}: {
  test_input_matrix: number[][];
  train_dataset: {
    dataset: {
      input_matrix: number[][];
      output_matrix: number[][];
      instances: {
        instance_name: string;
        input_instance_matrix: number[][];
        output_instance_matrix: number[][];
        implementation: string;
        input_value: object;
        output_value: object;
      }[];
    }[];
    declarations: string[];
  };
  input_instances: {
    input_instance_matrix: number[][];
    output_instance_matrix: number[][];
    input_value: object;
    output_value: object;
  }[];
  input_instance_matrix: number[][];
  input_value: object;
  output_value: object;
}) {
  const { x, y } = output_value as { x: number; y: number };

  terminal.step("Composing the output object");

  const { output_instance_matrix } = await llm.generate<{
    output_instance_matrix: number[][];
  }>({
    messages: [
      {
        role: "assistant",
        content: JSON.stringify({
          train_dataset: train_dataset.dataset.map(({ instances }) => ({
            instances: instances.map(
              ({
                input_value,
                input_instance_matrix,
                output_value,
                output_instance_matrix,
              }) => ({
                input_value,
                input_instance_matrix: Matrix.crop(input_instance_matrix),
                output_value,
                output_instance_matrix: Matrix.crop(output_instance_matrix),
              })
            ),
          })),
          input_instances: input_instances.map(
            ({
              input_value,
              input_instance_matrix,
              output_value,
              output_instance_matrix,
            }) => ({
              input_value,
              input_instance_matrix: Matrix.crop(input_instance_matrix),
              output_value,
              output_instance_matrix: Matrix.crop(output_instance_matrix),
            })
          ),
        }),
      },
      {
        role: "user",
        content: JSON.stringify({
          input_value,
          input_instance_matrix: Matrix.crop(input_instance_matrix),
          output_value,
        }),
      },
    ],
    response_format: responseFormat.create(
      "visualizer_output_instance_matrix",
      {
        output_instance_matrix: responseFormat.grid(),
      }
    ),
  });

  terminal.grid("Output crop", output_instance_matrix);

  const output_instance_padded_matrix = Matrix.pad(
    output_instance_matrix,
    test_input_matrix.length,
    test_input_matrix[0].length,
    x,
    y
  );

  terminal.grid("Placed output", output_instance_padded_matrix);

  return { output_instance_matrix: output_instance_padded_matrix };
}
