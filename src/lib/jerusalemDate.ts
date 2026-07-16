import { toDateString } from "./operationalDay";

const JERUSALEM_LATITUDE = 31.7683;
const JERUSALEM_LONGITUDE = 35.2137;
const OFFICIAL_SUNSET_ZENITH = 90.833;

function normalizeDegrees(value: number): number {
  return ((value % 360) + 360) % 360;
}

function normalizeHours(value: number): number {
  return ((value % 24) + 24) % 24;
}

function degreesToRadians(value: number): number {
  return (value * Math.PI) / 180;
}

function radiansToDegrees(value: number): number {
  return (value * 180) / Math.PI;
}

function dayOfYear(date: Date): number {
  const start = Date.UTC(date.getFullYear(), 0, 0);
  const current = Date.UTC(date.getFullYear(), date.getMonth(), date.getDate());
  return Math.floor((current - start) / 86_400_000);
}

/** Astronomical sunset for Jerusalem using NOAA's sunrise/sunset algorithm. */
export function jerusalemSunset(date: Date): Date {
  const longitudeHour = JERUSALEM_LONGITUDE / 15;
  const approximateTime = dayOfYear(date) + (18 - longitudeHour) / 24;
  const meanAnomaly = 0.9856 * approximateTime - 3.289;
  const trueLongitude = normalizeDegrees(
    meanAnomaly +
      1.916 * Math.sin(degreesToRadians(meanAnomaly)) +
      0.02 * Math.sin(degreesToRadians(2 * meanAnomaly)) +
      282.634,
  );

  let rightAscension = normalizeDegrees(
    radiansToDegrees(
      Math.atan(0.91764 * Math.tan(degreesToRadians(trueLongitude))),
    ),
  );
  rightAscension +=
    Math.floor(trueLongitude / 90) * 90 -
    Math.floor(rightAscension / 90) * 90;
  rightAscension /= 15;

  const sinDeclination =
    0.39782 * Math.sin(degreesToRadians(trueLongitude));
  const cosDeclination = Math.cos(Math.asin(sinDeclination));
  const cosineHourAngle =
    (Math.cos(degreesToRadians(OFFICIAL_SUNSET_ZENITH)) -
      sinDeclination *
        Math.sin(degreesToRadians(JERUSALEM_LATITUDE))) /
    (cosDeclination * Math.cos(degreesToRadians(JERUSALEM_LATITUDE)));

  const hourAngle =
    radiansToDegrees(Math.acos(Math.max(-1, Math.min(1, cosineHourAngle)))) /
    15;
  const localMeanTime =
    hourAngle + rightAscension - 0.06571 * approximateTime - 6.622;
  const utcHours = normalizeHours(localMeanTime - longitudeHour);

  const utcMidnight = Date.UTC(
    date.getFullYear(),
    date.getMonth(),
    date.getDate(),
  );
  return new Date(utcMidnight + utcHours * 3_600_000);
}

/** Civil date whose Hebrew date is currently in effect in Jerusalem. */
export function getJerusalemHebrewDate(now = new Date()): Date {
  const effective = new Date(now);
  if (now >= jerusalemSunset(now)) {
    effective.setDate(effective.getDate() + 1);
  }
  return effective;
}

export function getJerusalemHebrewDateString(now = new Date()): string {
  return toDateString(getJerusalemHebrewDate(now));
}
