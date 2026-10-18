import * as nucleoid from "./lib/nucleoid";
import * as Matrix from "./lib/Matrix";
import * as llm from "./lib/llm";
import type { Grid, TestInstance, TrainingDataset } from "./types";

export async function instances({
  train_dataset,
  test_input_matrix,
}: {
  train_dataset: TrainingDataset;
  test_input_matrix: Grid;
}): Promise<{ instances: TestInstance[] }> {
  console.log("Analyzing test_input_matrix...");
  console.log("");

  console.debug("test_input_matrix:");
  Matrix.toString(test_input_matrix);
  console.debug("");

  const { instances } = await llm.generate<{
    instances: TestInstance[];
  }>({
    messages: [
      {
        role: "user",
        content: `
          train_dataset:
          ${JSON.stringify(train_dataset)}
          input_matrix:
          ${JSON.stringify(test_input_matrix)}
          json_format:
          {
            "instances": [{
              "input_instance_matrix": <INSTANCE_MATRIX>
            }]
          }
        `,
      },
    ],
  });

  console.debug(`${instances.length} instances`);
  instances.forEach(({ input_instance_matrix }) => {
    Matrix.toString(input_instance_matrix);
    console.debug("--");
  });

  return { instances };
}

export async function value({
  instance_name,
  train_dataset,
  input_instance_matrix,
}: {
  instance_name: string;
  train_dataset: TrainingDataset;
  input_instance_matrix: Grid;
}): Promise<{ input_value: unknown; output_value: unknown }> {
  console.log("Calculating test value...");
  const { input_code } = await llm.generate<{ input_code: string }>({
    messages: [
      {
        role: "user",
        content: `
          instance_name:
          ${instance_name}
          train_dataset:
          ${JSON.stringify(train_dataset)}
          input_instance_matrix:
          ${JSON.stringify(input_instance_matrix)}
          json_format:
          { "input_code": <INPUT_CODE> }
        `,
      },
    ],
  });

  console.log("Creating test instance in Nucleoid...");
  const input_value = await nucleoid.run(
    [train_dataset.declarations[0], input_code].join("\n\n")
  );
  const output_value = await nucleoid.run(
    [...train_dataset.declarations, input_code].join("\n\n")
  );

  console.debug("input_code:");
  console.debug(input_code);
  console.debug("input_value:");
  console.debug(input_value);
  console.debug("output_value:");
  console.debug(output_value);

  return { input_value, output_value };
}

export async function output_instance_matrix({
  test_input_matrix,
  train_dataset,
  input_instance_matrix,
  input_value,
  output_value,
}: {
  test_input_matrix: Grid;
  train_dataset: TrainingDataset;
  input_instance_matrix: Grid;
  input_value: unknown;
  output_value: unknown;
}): Promise<{ output_instance_matrix: Grid }> {
  console.log("Extracting test output_instance_matrix...");
  const { output_instance_matrix } = await llm.generate<{
    output_instance_matrix: Grid;
  }>({
    document: false,
    messages: [
      {
        role: "user",
        content: `
          instances:
          ${JSON.stringify(
            train_dataset.dataset.map(({ instances }) => ({
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
            }))
          )}
          input_instance_matrix:
          ${JSON.stringify(input_instance_matrix)}
          input_value:
          ${JSON.stringify(input_value)}
          output_value:
          ${JSON.stringify(output_value)}
          json_format:
          {
            "output_instance_matrix": <OUTPUT_INSTANCE_MATRIX>
          }
        `,
      },
    ],
  });

  console.debug("output_instance_matrix:");
  Matrix.toString(output_instance_matrix);

  return { output_instance_matrix };
}
