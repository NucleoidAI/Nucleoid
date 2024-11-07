# nuc-arc

## ARC Benchmark

The ARC benchmark provides a dataset and evaluation benchmark designed to test the reasoning abilities of AI models. The Abstraction and Reasoning Corpus (ARC) benchmark, introduced by François Chollet, tests an AI's ability to demonstrate general intelligence rather than task-specific performance. Unlike traditional benchmarks that focus on training models on large, fixed datasets, ARC assesses how well an AI system can generalize, reason abstractly, and solve novel problems using limited information—skills akin to human cognitive abilities. It challenges AI to learn and adapt without task-specific training, emphasizing flexibility and the capacity to understand and apply abstract relationships. Designed with problems that a human can solve using common sense and basic reasoning, ARC remains difficult for AI, highlighting the gap between current models and true human-like cognitive capabilities. This benchmark is seen as a significant step toward developing Artificial General Intelligence (AGI), as it requires AI systems to exhibit foundational skills like inductive reasoning, analogy-making, and adaptability, pushing AI research toward more human-like problem-solving abilities.

https://arcprize.org/

![ARC Puzzle Example](https://github.com/user-attachments/assets/41701a8e-5639-4f35-96ae-f4815dbd59cc)

## Nucleoid Approach

Nucleoid aka `nuc` approaches Neuro-Symbolic AI with introducing an intermediate language. Briefly, Nucleoid is a declarative, logic-based, contextual runtime that tracks each statement in declarative syntax and dynamically creates relationships between both logic and data statements in the knowledge graph to used in decision-making and problem-solving process.

> Essential Intelligence is integration of pattern, data and logic.

This concept is also introduced in *Thinking, Fast and Slow* by Daniel Kahneman, where System 1 operates through pattern recognition, while System 2 applies logical reasoning. Data acts as the bridge, enabling collaboration between these systems to yield insights based on both probabilistic and deterministic information. However, the real challenge lies in enabling effective collaboration between the two systems so they can understand and support one another.

We've found that using an intermediate, ubiquitous language is highly effective beyond NLP and CoT methods, as it allows both pattern recognition and logical reasoning to be represented through 5GL, facilitating seamless bidirectional communication between the systems.

### Process Flow

There are 2 sections in our approach: analysis and visualization. In analysis phase, the AI system aims generalize patterns and identifies instances in order to use in actual test, and in visualization, the extracted abstraction is applied to given test input.

> :zap: Instead of prompt engineering, all communications with LLMs are made thru `nuc` language

![ARC_Flow](https://github.com/user-attachments/assets/0b5132cb-8269-461d-b27b-2b84ec1dd640)

#### Analysis

##### 1. Find patterns and Represent in `nuc` lang

The LLM finds pattern for given training input and output data and creates declarations in `nuc` lang.

```
{
  "declarations": [
    "'use declarative'; class Obj { constructor(vertex, vertical_line_segment, horizontal_line_segment) { this.shape = 'unspecified'; this.vertex = vertex; this.vertical_line_segment = vertical_line_segment; this.horizontal_line_segment = horizontal_line_segment; this.extension = 0; }}",
    "'use declarative'; if($Obj.vertical_line_segment === 8 && $Obj.horizontal_line_segment === 8 && $Obj.vertex === 8) { $Obj.shape = 'label1234'; $Obj.extension = 1; }",
    ...
  ]
}
```

##### 2. Initialize Nucleoid session

Nucleoid session is initialized with the declarations in order to use for identified instances later on.

##### 3. Extract instances

Based on declarations in `nuc` lang, the LLM extracts instances.

```
{
  "instances": [
    {
      "input_object": {
        "x_position": 4,
        "y_position": 0,
        "object_matrix": [
          [8, 8],
          [0, 8]
        ]
      },
      "output_object": {
        "x_position": 4,
        "y_position": 0,
        "object_matrix": [
          [8, 8],
          [1, 8]
        ]
      }
    }
  ]
  ...
}
```

##### 4. Create instances in Nucleoid session

The LLM also represents identified instances in `nuc` lang. This step is crucial because this completes **knowledge packet**, which contains visual as well as logical representation of the instances.

```
{
  "instances": [
    {
      "input_object": {
        "x_position": 4,
        "y_position": 0,
        "object_matrix": [
          [8, 8],
          [0, 8]
        ]
      },
      "output_object": {
        "x_position": 4,
        "y_position": 0,
        "object_matrix": [
          [8, 8],
          [1, 8]
        ]
      },
      "input_code": "'use imperative'; var obj0 = new Obj(8, 8, 8); obj0;",
      "output_value": {
        "id": "obj0",
        "shape": "label1234",
        "vertex": 8,
        "vertical_line_segment": 8,
        "horizontal_line_segment": 8,
        "extension": 1,
      }
    }
  ]
  ...
}
```

#### Visualization

##### 5. Extract instances from Test Input

This step is slight different then from extracting instances in analysis because now the LLM has to use instances list from previous step as a reference.

##### 6. Initialize Nucleoid session for test input

In order not to mix up instances with training dataset, new Nucleoid session required, but the same declarations is applied.

##### 7. Create test instances in Nucleoid session

For identified instances, the LLM represents instances in `nuc` lang, this is particularly similar to the step in analysis phase.

##### 8. Visualize output instance

For each input instance in test, the LLM generates output instance since it has all information needed from **knowledge packets**.

Finally, all instances can be merged for test result.
