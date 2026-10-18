import * as Matrix from "./Matrix";
import * as nucleoid from "./nucleoid";
import * as llm from "./llm";
import type { AnalyzedInstance, Grid, TrainingInput } from "../types";

export async function declarations({
  train_dataset,
}: {
  train_dataset: TrainingInput;
}): Promise<{ declarations: string[] }> {
  console.log("Analyzing declarations...");

  const { declarations } = await llm.generate<{ declarations: string[] }>({
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

export async function instances({
  declarations,
  input_matrix,
  output_matrix,
}: {
  declarations: string[];
  input_matrix: Grid;
  output_matrix: Grid;
}): Promise<{ instances: AnalyzedInstance[] }> {
  console.log("Extracting instances...");

  const { instances } = await llm.generate<{
    instances: AnalyzedInstance[];
  }>({
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
}: {
  instance_name: string;
  declarations: string[];
  input_instance_matrix: Grid;
  output_instance_matrix: Grid;
}): Promise<{ input_code: string; output_value: unknown }> {
  console.log("Calculating value...");

  const { input_code } = await llm.generate<{ input_code: string }>({
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
  const output_value = await nucleoid.run(
    [...declarations, input_code].join("\n\n")
  );

  console.debug("input_code:");
  console.debug(input_code);
  console.debug("output_value:");
  console.debug(output_value);

  return { input_code, output_value };
}
