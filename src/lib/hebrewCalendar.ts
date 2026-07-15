import { formatHebrewDay, formatHebrewYear } from "./hebrewDate";
import { parseDateString, toDateString } from "./operationalDay";

interface HebrewMonthParts {
  day: number;
  monthKey: string;
  year: number;
}

interface HebrewCalendarCell {
  dateStr: string;
  hebrewDay: number;
}

const hebrewDateFormatter = new Intl.DateTimeFormat("he-u-ca-hebrew", {
  day: "numeric",
  month: "long",
  year: "numeric",
});

const hebrewMonthNameFormatter = new Intl.DateTimeFormat("he-u-ca-hebrew", {
  month: "long",
});

function getHebrewMonthKey(date: Date): string {
  const parts = hebrewDateFormatter.formatToParts(date);
  const month = parts.find((part) => part.type === "month")?.value ?? "";
  const year = parts.find((part) => part.type === "year")?.value ?? "";
  return `${month}-${year}`;
}

export function getHebrewMonthParts(date: Date): HebrewMonthParts {
  const parts = hebrewDateFormatter.formatToParts(date);

  return {
    day: Number.parseInt(
      parts.find((part) => part.type === "day")?.value ?? "1",
      10,
    ),
    monthKey: getHebrewMonthKey(date),
    year: Number.parseInt(
      parts.find((part) => part.type === "year")?.value ?? "0",
      10,
    ),
  };
}

export function formatHebrewMonthYear(date: Date): string {
  const { year } = getHebrewMonthParts(date);
  const monthName = hebrewMonthNameFormatter.format(date);
  return `${monthName} ${formatHebrewYear(year)}`;
}

function getHebrewMonthStart(date: Date): Date {
  let current = new Date(date);
  const targetKey = getHebrewMonthKey(current);

  while (true) {
    const previous = new Date(current);
    previous.setDate(previous.getDate() - 1);
    const previousKey = getHebrewMonthKey(previous);

    if (previousKey !== targetKey) {
      return current;
    }

    current = previous;
  }
}

function getHebrewMonthEnd(monthStart: Date): Date {
  const targetKey = getHebrewMonthKey(monthStart);
  let current = new Date(monthStart);

  while (true) {
    const next = new Date(current);
    next.setDate(next.getDate() + 1);
    const nextKey = getHebrewMonthKey(next);

    if (nextKey !== targetKey) {
      return current;
    }

    current = next;
  }
}

export function shiftHebrewMonth(dateStr: string, delta: number): string {
  const anchor = parseDateString(dateStr);
  let monthStart = getHebrewMonthStart(anchor);

  if (delta > 0) {
    for (let step = 0; step < delta; step += 1) {
      const monthEnd = getHebrewMonthEnd(monthStart);
      monthStart = new Date(monthEnd);
      monthStart.setDate(monthStart.getDate() + 1);
    }
  } else if (delta < 0) {
    for (let step = 0; step < -delta; step += 1) {
      const previous = new Date(monthStart);
      previous.setDate(previous.getDate() - 1);
      monthStart = getHebrewMonthStart(previous);
    }
  }

  return toDateString(monthStart);
}

export function getHebrewMonthGrid(dateStr: string): Array<HebrewCalendarCell | null> {
  const anchor = parseDateString(dateStr);
  const monthStart = getHebrewMonthStart(anchor);
  const monthEnd = getHebrewMonthEnd(monthStart);
  const cells: Array<HebrewCalendarCell | null> = [];

  const leadingEmpty = monthStart.getDay();
  for (let index = 0; index < leadingEmpty; index += 1) {
    cells.push(null);
  }

  const current = new Date(monthStart);
  while (current <= monthEnd) {
    cells.push({
      dateStr: toDateString(current),
      hebrewDay: getHebrewMonthParts(current).day,
    });
    current.setDate(current.getDate() + 1);
  }

  while (cells.length % 7 !== 0) {
    cells.push(null);
  }

  return cells;
}

export function formatHebrewDayInCell(day: number): string {
  return formatHebrewDay(day);
}

export function getHebrewMonthRange(dateStr: string): {
  start: string;
  end: string;
} {
  const anchor = parseDateString(dateStr);
  const monthStart = getHebrewMonthStart(anchor);
  const monthEnd = getHebrewMonthEnd(monthStart);

  return {
    start: toDateString(monthStart),
    end: toDateString(monthEnd),
  };
}
