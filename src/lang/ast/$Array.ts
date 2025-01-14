import $Node from "./$Node";

class $Array extends $Node {
  elements: $Node[];

  constructor(elements) {
    super(elements);
    this.elements = elements;
  }

  generate(scope) {
    return `[${this.elements.map((el) => el.generate(scope)).join(",")}]`;
  }
}

export default $Array;
