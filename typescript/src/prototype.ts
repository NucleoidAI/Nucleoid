declare global {
  interface DateConstructor {
    now: { value: boolean };
  }
  interface Array<T> {
    push: { write: boolean };
  }
}

// Assigning custom properties
(Date.now as any).value = true;
(Array.prototype.push as any).write = true;
