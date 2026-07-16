export function getOperationalDate(now = new Date()): string {
  // Calendar-facing Gregorian dates change at civil midnight. Backend
  // playback may still group post-midnight slots with the prior operating day.
  return toDateString(now);
}

export function toDateString(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function formatGregorianDayMonth(dateStr: string): string {
  const [, month, day] = dateStr.split("-");
  return `${day}.${month}`;
}

export function parseDateString(value: string): Date {
  return new Date(`${value}T12:00:00`);
}

/** Next calendar day as YYYY-MM-DD. */
export function nextDateString(dateStr: string): string {
  const date = parseDateString(dateStr);
  date.setDate(date.getDate() + 1);
  return toDateString(date);
}
