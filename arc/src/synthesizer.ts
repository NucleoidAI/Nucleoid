import * as llm from "./lib/llm.ts";
import * as Matrix from "./lib/Matrix";

const FIELD_GUIDE = `
- "dataset" holds the existing examples. Each pairs an "input_matrix" with the "output_matrix" that the rule produces from it.
- Each example's "instances" lists its instances, each pairing an "input_instance_matrix" with its "output_instance_matrix". They relate to the example's full matrices by a fixed convention (how instances are found, ordered and cropped); work that convention out from the existing examples and follow it exactly.
- "declarations" and "output_canvas" are fixed for this task. Every new example must be valid under them. Do not change them or repeat them in your answer.
`.trim();

async function synthesize({ train_dataset }) {
  const count = train_dataset.length;

  const { result } = await llm.generate({
    few_shot: null,
    messages: [
      {
        role: "assistant",
        content: `Below is the data for a matrix-transformation task. Each example pairs an input matrix with the output matrix that one hidden rule produces from it, and the same rule holds for every example.

        <current_dataset>
        ${JSON.stringify({
          ...train_dataset,
          dataset: train_dataset.dataset.map(
            ({ input_matrix, output_matrix, instances }) => ({
              input_matrix,
              output_matrix,
              instances: instances.map((instance) => ({
                ...instance,
                input_instance_crop_matrix: Matrix.crop(
                  instance.input_instance_matrix
                ),
                output_instance_crop_matrix: Matrix.crop(
                  instance.output_instance_matrix
                ),
              })),
            })
          ),
        })}
        </current_dataset>
        
        How to read it:
        ${FIELD_GUIDE}
        
        Your job: write ${
          count === 1
            ? "1 new example that follows"
            : `${count} new examples that follow`
        } exactly the same rule. New examples are added to the training data, so one whose output breaks the rule does more harm than a missing one.
        
        Work in this order:
        1. Infer the rule and write it as a sequence of steps, precise enough that someone could apply it by hand to an input they have never seen, including how the instances are derived. Check it against every existing example and revise it until it reproduces all of them exactly.
        2. List the invariants: what every existing example has in common that the rule relies on, plus anything "declarations" and "output_canvas" impose (allowed values, dimensions, how many of something appear, what never touches or overlaps). If you are unsure whether the rule depends on a property, treat it as an invariant. Every new example must keep all of them.
        3. Plan each new example in a sentence or two: what its input contains, how it differs from the existing examples and from your other new examples, and what output the rule gives for it.
        4. Write the examples. Build each input first, then produce its output by applying the rule step by step, copying exactly any cell the rule leaves unchanged. Then derive its instances with the same convention as the existing examples.
        
        A good new example is:
        - Correct: its output is exactly what the rule produces from its input. If you are not certain an example is correct, leave it out; returning fewer examples is better than including a wrong one.
        - New: not a near-copy of an existing example, such as the same layout shifted, flipped or with values swapped. Vary whatever the rule does not depend on, which may include dimensions, positions, sizes, counts and values, while keeping every invariant.
        - Informative: it covers a case the existing examples don't (an edge or corner position, a smallest or largest case, a different number of instances) or separates the real rule from a plausible wrong one, but only where the rule clearly decides the output. Avoid inputs the rule leaves open, such as ties it gives no way to break.
        - Well-formed: every matrix is rectangular (all rows the same length) and uses only allowed values.
        
        Reply with one JSON object and nothing else: no prose before or after it and no code fences. Give its keys in this order:
        - "rule": an array of strings, one step of the rule each.
        - "invariants": an array of strings, one invariant each.
        - "plan": an array of strings, one per new example, in the same order as "result".
        - "result": an array of the new examples. Each has exactly the same keys, nesting and value types as the entries of "dataset" above: "input_matrix", "output_matrix", and "instances", whose entries have "input_instance_matrix" and "output_instance_matrix". Write matrices in the same encoding the existing examples use; if theirs are arrays of arrays of numbers, yours are too, not strings.`,
      },
      {
        role: "user",
        content: `
          synthesize:
          ${JSON.stringify({
            declarations: train_dataset.declarations,
            output_canvas: train_dataset.output_canvas,
            dataset: [
              {
                input_matrix: "<INPUT_MATRIX>",
                output_matrix: "<OUTPUT_MATRIX>",
                instances: [
                  {
                    input_instance_crop_matrix: "<INPUT_INSTANCE_CROP_MATRIX>",
                    input_instance_matrix: "<INPUT_INSTANCE_MATRIX>",
                    output_instance_crop_matrix:
                      "<OUTPUT_INSTANCE_CROP_MATRIX>",
                    output_instance_matrix: "<OUTPUT_INSTANCE_MATRIX>",
                  },
                ],
              },
            ],
          })}
          json_format:
          {
            "result": [
              {
                "instances": [
                  {
                    "input_instance_crop_matrix": "<INPUT_INSTANCE_CROP_MATRIX>",
                    "input_instance_matrix": "<INPUT_INSTANCE_MATRIX>",
                    "output_instance_crop_matrix": "<OUTPUT_INSTANCE_CROP_MATRIX>",
                    "output_instance_matrix": "<OUTPUT_INSTANCE_MATRIX>",
                  },
                ],
                "input_matrix": "<INPUT_MATRIX>",
                "output_matrix": "<OUTPUT_MATRIX>",
              },
            ]
          }
          `,
      },
    ],
  });

  return result;
}

export default { synthesize };
