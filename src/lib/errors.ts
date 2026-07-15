/**
 * Extracts a human-readable message from an unknown thrown value, falling back
 * to a caller-provided default when the value is not a standard `Error`.
 */
export function errMsg(err: unknown, fallback: string): string {
  return err instanceof Error ? err.message : fallback;
}
