const alphanumericChars =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const alphaChars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

function random(length = 16, alphanumeric = false) {
  let result = alphaChars.charAt(Math.floor(Math.random() * alphaChars.length));

  let chars;

  if (alphanumeric) {
    chars = alphanumericChars;
  } else {
    chars = alphaChars;
  }

  for (let i = 1; i < length; i++) {
    const randomIndex = Math.floor(Math.random() * chars.length);
    result += chars.charAt(randomIndex);
  }

  return result;
}

module.exports = random;
