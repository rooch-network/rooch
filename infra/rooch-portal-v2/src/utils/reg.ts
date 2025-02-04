export function isNumber(value: string) {
  return /^\d*\.?\d*$/.test(value);
}
