import { useEffect, useState } from "react";
import { formatHebrewDate } from "../../lib/hebrewDate";

function Clock({ date }: { date: Date }) {
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  const seconds = String(date.getSeconds()).padStart(2, "0");

  return (
    <span className="font-bold tabular-nums">
      {hours}:{minutes}
      <span className="text-[0.7em] font-semibold opacity-90">:{seconds}</span>
    </span>
  );
}

function formatWeekday(date: Date): string {
  return date.toLocaleDateString("he-IL", { weekday: "long" });
}

function formatGregorianDate(date: Date): string {
  const day = String(date.getDate()).padStart(2, "0");
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const year = date.getFullYear();
  return `${day}/${month}/${year}`;
}

function Separator() {
  return (
    <span className="text-teal-400" aria-hidden>
      |
    </span>
  );
}

export default function DateTimeHeader() {
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    const intervalId = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(intervalId);
  }, []);

  return (
    <div
      className="flex flex-wrap items-center justify-center gap-x-3 gap-y-0.5 text-sm text-teal-50"
      dir="rtl"
    >
      <Clock date={now} />
      <Separator />
      <span>{formatWeekday(now)}</span>
      <Separator />
      <span className="tabular-nums">{formatGregorianDate(now)}</span>
      <Separator />
      <span className="text-teal-100">{formatHebrewDate(now)}</span>
    </div>
  );
}
