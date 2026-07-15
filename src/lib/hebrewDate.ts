const ONES = ["", "א", "ב", "ג", "ד", "ה", "ו", "ז", "ח", "ט"];
const TENS = ["", "י", "כ", "ל", "מ", "נ", "ס", "ע", "פ", "צ"];

function toHebrewLetters(value: number): string {
  if (value <= 0 || value > 999) {
    return String(value);
  }

  let number = value;
  let result = "";

  if (number >= 100) {
    const hundreds = Math.floor(number / 100);
    if (hundreds === 4) {
      result += "ת";
    } else if (hundreds <= 3) {
      result += ["", "ק", "ר", "ש"][hundreds];
    } else {
      result += "ת" + ["", "ק", "ר", "ש", "ת"][hundreds - 4];
    }
    number %= 100;
  }

  if (number === 15) {
    return result + "טו";
  }
  if (number === 16) {
    return result + "טז";
  }

  if (number >= 10) {
    result += TENS[Math.floor(number / 10)];
    number %= 10;
  }
  if (number > 0) {
    result += ONES[number];
  }

  return result;
}

function addHebrewPunctuation(letters: string): string {
  if (letters.length <= 1) {
    return `${letters}\u05F3`;
  }
  return `${letters.slice(0, -1)}\u05F4${letters.slice(-1)}`;
}

export function formatHebrewDay(day: number): string {
  return addHebrewPunctuation(toHebrewLetters(day));
}

export function formatHebrewYear(year: number): string {
  return addHebrewPunctuation(toHebrewLetters(year % 1000));
}

const hebrewDatePartsFormatter = new Intl.DateTimeFormat("he-u-ca-hebrew", {
  day: "numeric",
  month: "short",
  year: "numeric",
});

export function formatHebrewDate(date: Date): string {
  const parts = hebrewDatePartsFormatter.formatToParts(date);

  const day = Number.parseInt(
    parts.find((part) => part.type === "day")?.value ?? "1",
    10,
  );
  const month = parts.find((part) => part.type === "month")?.value ?? "";
  const year = Number.parseInt(
    parts.find((part) => part.type === "year")?.value ?? "0",
    10,
  );

  return `${formatHebrewDay(day)} ${month} ${formatHebrewYear(year)}`;
}
