const alphanumericChars: string =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const alphaChars: string =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

function random(length: number = 16, alphanumeric: boolean = false): string {
  let result: string = alphaChars.charAt(
    Math.floor(Math.random() * alphaChars.length)
  );

  let chars: string;

  if (alphanumeric) {
    chars = alphanumericChars;
  } else {
    chars = alphaChars;
  }

  for (let i: number = 1; i < length; i++) {
    const randomIndex: number = Math.floor(Math.random() * chars.length);
    result += chars.charAt(randomIndex);
  }

  return result;
}

export { random };
