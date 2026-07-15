export type EmergencyAlertCategoryId =
  | "pre-alert"
  | "red-alert"
  | "hostile-aircraft"
  | "end"
  | "unconfigured";

export interface EmergencyMessageType {
  id: EmergencyAlertCategoryId;
  label: string;
  description: string;
  accent: {
    border: string;
    bg: string;
    icon: string;
  };
}

export const EMERGENCY_MESSAGE_TYPES: EmergencyMessageType[] = [
  {
    id: "pre-alert",
    label: "התראה מקדימה",
    description: "התרעות לפני צבע אדום",
    accent: {
      border: "border-amber-200",
      bg: "bg-amber-50/80",
      icon: "text-amber-600",
    },
  },
  {
    id: "red-alert",
    label: "צבע אדום",
    description: "ירי רקטות וטילים",
    accent: {
      border: "border-red-200",
      bg: "bg-red-50/80",
      icon: "text-red-600",
    },
  },
  {
    id: "hostile-aircraft",
    label: "חדירת כלי טייס עוין",
    description: "חדירת כלי טיס עוין לאזור",
    accent: {
      border: "border-orange-200",
      bg: "bg-orange-50/80",
      icon: "text-orange-600",
    },
  },
  {
    id: "end",
    label: "סיום",
    description: "סיום אירוע והנחיות יציאה",
    accent: {
      border: "border-teal-200",
      bg: "bg-teal-50/60",
      icon: "text-teal-600",
    },
  },
  {
    id: "unconfigured",
    label: "הודעה לא מוגדרת",
    description: "התרעות שאינן משויכות לאף סוג מוכר",
    accent: {
      border: "border-slate-200",
      bg: "bg-slate-50/80",
      icon: "text-slate-500",
    },
  },
];
