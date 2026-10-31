import * as Matrix from "./lib/Matrix";
import * as terminal from "./lib/terminal";

export function start({ version, train, test_input_matrix }) {
  terminal.header({ version });
  terminal.puzzle({ train, testInput: test_input_matrix });
}

export function analysis() {
  terminal.section(
    1,
    "Analyze",
    "Infer reusable objects and transformation rules"
  );
}

export function inferring_declarations() {
  terminal.step("Inferring declarations");
}

export function declarations_inferred({ declarations, output_canvas }) {
  terminal.code("Declarations", declarations);
  terminal.step(`Output canvas ${output_canvas}`);
}

export function training_pair({ index, total }) {
  terminal.step(`Training pair ${index + 1} / ${total}`);
}

export function segmenting_training_objects() {
  terminal.step("Segmenting training objects");
}

export function training_objects_segmented({ instances }) {
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
}

export function constructing_instance({ instance_name }) {
  terminal.step(`Constructing ${instance_name}`);
}

export function evaluating_rules() {
  terminal.step("Evaluating declarative rules");
}

export function instance_constructed({
  instance_name,
  implementation,
  input_instance_matrix,
  output_instance_matrix,
  input_value,
}) {
  terminal.grid("Input crop", Matrix.crop(input_instance_matrix));
  terminal.grid("Output crop", Matrix.crop(output_instance_matrix));
  terminal.code(`Implementation · ${instance_name}`, implementation);
  terminal.value("Input state", input_value);
}

export function visualization() {
  terminal.section(
    2,
    "Visualize",
    "Apply the learned world model to the test input"
  );
}

export function segmenting_test_input({ test_input_matrix }) {
  terminal.step("Segmenting the test input");
  terminal.grid("Test input", test_input_matrix);
}

export function test_input_segmented({ instances }) {
  terminal.step(`${instances.length} objects identified`);
  instances.forEach(({ input_instance_matrix }, index: number) => {
    terminal.grid(`Test object ${index + 1}`, input_instance_matrix);
  });
}

export function synthesizing_instance({ instance_name }) {
  terminal.step(`Synthesizing ${instance_name}`);
}

export function instance_synthesized({ instance_name, implementation }) {
  terminal.code(`Implementation · ${instance_name}`, implementation);
}

export function executing_transformation() {
  terminal.step("Executing the object transformation");
}

export function transformation_executed({ input_value }) {
  terminal.value("Input state", input_value);
}

export function composing_output() {
  terminal.step("Composing the output object");
}

export function output_composed({ output_instance_matrix }) {
  terminal.grid("Output crop", output_instance_matrix);
}

export function output_placed({ output_instance_padded_matrix }) {
  terminal.grid("Placed output", output_instance_padded_matrix);
}

export function comparison({ test_output_matrix, result_matrix }) {
  terminal.section(3, "Compare", "Expected and predicted canvases");
  terminal.grid("Expected", test_output_matrix);
  terminal.grid("Result", result_matrix);
}

export function summary({ difference, duration }) {
  terminal.summary(difference, duration);
}

export function usage({ prompt_tokens, completion_tokens }) {
  terminal.usage({ prompt_tokens, completion_tokens });
}

export function failure(error) {
  terminal.failure(error);
}
