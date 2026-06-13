/** Cents (integer) -> "$12.34". Handles negatives for short positions. */
export function formatDollars(cents: number): string {
  const sign = cents < 0 ? "-" : "";
  const abs = Math.abs(cents);
  return `${sign}$${(abs / 100).toFixed(2)}`;
}

/** A contract price 1..99 -> "60¢". Prices are already in cents. */
export function formatPrice(price: number): string {
  return `${price}¢`;
}

/** A price 1..99 as an implied probability -> "60%". */
export function formatProbability(price: number): string {
  return `${price}%`;
}
