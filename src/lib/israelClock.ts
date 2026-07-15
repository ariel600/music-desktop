export type IsraelClockSeason = "winter" | "summer";

const JERUSALEM_TIME_ZONE = "Asia/Jerusalem";

function getJerusalemUtcOffsetMinutes(date: Date): number {
  const timeZoneName = new Intl.DateTimeFormat("en-US", {
    timeZone: JERUSALEM_TIME_ZONE,
    timeZoneName: "shortOffset",
  })
    .formatToParts(date)
    .find((part) => part.type === "timeZoneName")?.value;

  if (!timeZoneName) {
    return 120;
  }

  const match = timeZoneName.match(/([+-])(\d{1,2})(?::?(\d{2}))?/);
  if (!match) {
    return 120;
  }

  const sign = match[1] === "-" ? -1 : 1;
  const hours = Number(match[2]);
  const minutes = Number(match[3] ?? "0");
  return sign * (hours * 60 + minutes);
}

export function getActiveIsraelClockSeason(
  date: Date = new Date(),
): IsraelClockSeason {
  return getJerusalemUtcOffsetMinutes(date) >= 180 ? "summer" : "winter";
}
