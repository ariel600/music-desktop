interface ToggleSwitchProps {
  checked: boolean;
  onChange?: (checked: boolean) => void;
  readOnly?: boolean;
  disabled?: boolean;
  id?: string;
    offLabel?: string;
    onLabel?: string;
}

export default function ToggleSwitch({
  checked,
  onChange,
  readOnly = false,
  disabled = false,
  id,
  offLabel = "כבוי",
  onLabel = "מופעל",
}: ToggleSwitchProps) {
  const isInteractive = !readOnly && !disabled;

  return (
    <div className="flex items-center gap-2">
      <span
        className={`text-xs font-medium transition-colors ${
          !checked ? "text-teal-900" : "text-teal-400"
        }`}
      >
        {offLabel}
      </span>

      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={checked}
        aria-readonly={readOnly}
        disabled={!isInteractive}
        onClick={isInteractive ? () => onChange?.(!checked) : undefined}
        className={`relative h-7 w-12 shrink-0 rounded-full transition-colors ${
          isInteractive
            ? "focus:outline-none focus-visible:ring-2 focus-visible:ring-teal-500 focus-visible:ring-offset-2"
            : "cursor-default"
        } disabled:cursor-default disabled:opacity-100 ${
          checked ? "bg-teal-600" : "bg-slate-300"
        }`}
      >
        <span
          className={`absolute top-0.5 h-6 w-6 rounded-full bg-white shadow transition-all ${
            checked ? "end-0.5" : "start-0.5"
          }`}
        />
      </button>

      <span
        className={`text-xs font-medium transition-colors ${
          checked ? "text-teal-900" : "text-teal-400"
        }`}
      >
        {onLabel}
      </span>
    </div>
  );
}
