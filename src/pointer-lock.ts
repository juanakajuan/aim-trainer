export type InputMode = "raw-requested" | "standard" | "not-captured";
export interface PointerLockTarget {
  requestPointerLock(options?: PointerLockOptions): Promise<void> | void;
}
export async function captureMouse(
  target: PointerLockTarget,
): Promise<InputMode> {
  try {
    await target.requestPointerLock({ unadjustedMovement: true });
    // Browsers can silently ignore options. This reports the request, not verified raw support.
    return "raw-requested";
  } catch (error: unknown) {
    if (!(
      typeof error === "object" &&
      error !== null &&
      "name" in error &&
      error.name === "NotSupportedError"
    ))
      throw error;
    await target.requestPointerLock();
    return "standard";
  }
}
