//REVISIT
type EventData = Record<string, string | number | boolean | null | undefined>;

let events: Array<{ name: string; data: string }> = [];

function event(name: string, data: EventData): void {
  events.push({ name, data: JSON.stringify(data) });
}

function list(): Array<{ name: string; data: string }> | undefined {
  if (events.length) {
    return events;
  }
  return undefined;
}

function clear(): void {
  events = [];
}

export { event, list, clear };
