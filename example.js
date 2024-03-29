export default function handler(message) {
  message = JSON.parse(message);
  let sum = 0;
  for (let i = 0; i < 100_000; i++) {
    sum += i;
  }
  return JSON.stringify({
    t: "performedSum",
    d: {
      sum,
      message,
    },
  });
}
