import DateTimeHeader from "./DateTimeHeader";

export default function TitleBar() {
  return (
    <header
      className="relative flex min-h-12 shrink-0 items-center bg-teal-800 px-4 py-2 text-teal-50"
      dir="rtl"
    >
      <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
        <DateTimeHeader />
      </div>

      <div className="relative z-10 flex items-center gap-2">
        <img
          src="/logo.png"
          alt=""
          className="h-8 w-8 rounded-lg object-cover shadow-sm"
        />
        <h1 className="text-lg font-bold tracking-tight">מערכת הודעות חכמה</h1>
      </div>
    </header>
  );
}
