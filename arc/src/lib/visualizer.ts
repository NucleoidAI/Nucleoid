import * as nucleoid from "./nucleoid";
import * as Matrix from "./Matrix";
import * as llm from "./llm";
import type { Grid, TestInstance, TrainingDataset } from "../types";

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
}): Promise<{ output_value: unknown }> {
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
  const output_value = await nucleoid.run(
    [...train_dataset.declarations, input_code].join("\n\n")
  );

  console.debug("input_code:");
  console.debug(input_code);
  console.debug("output_value:");
  console.debug(output_value);

  return { output_value };
}

export async function output_instance_matrix({
  test_input_matrix,
  result_matrix,
  train_dataset,
  input_instance_matrix,
  output_value,
}: {
  test_input_matrix: Grid;
  result_matrix: Grid;
  train_dataset: TrainingDataset;
  input_instance_matrix: Grid;
  output_value: unknown;
}): Promise<{ output_instance_matrix: Grid }> {
  console.log("Extracting test output_instance_matrix...");
  const { output_instance_matrix } = await llm.generate<{
    output_instance_matrix: Grid;
  }>({
    messages: [
      {
        role: "user",
        content: `
          instances:
          ${JSON.stringify(
            train_dataset.dataset.map(({ instances }) => ({
              instances: instances.map(
                ({
                  input_instance_matrix,
                  output_instance_matrix,
                  output_value,
                }) => ({
                  input_instance_matrix,
                  output_instance_matrix,
                  output_value,
                })
              ),
            }))
          )}
          input_instance_matrix:
          ${JSON.stringify(input_instance_matrix)}
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
