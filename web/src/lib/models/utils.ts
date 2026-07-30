import { DateTime } from "luxon";

export type CaptchaRequest = {
  captcha_id: string;
  captcha_answer: string;
};

const lifecycleAtKeys = new Set(["registration_at", "running_at", "review_at", "finished_at"]);

export function luxonReviver(key: string, value: unknown): unknown {
  if (lifecycleAtKeys.has(key)) {
    if (typeof value === "string") {
      return DateTime.fromISO(value);
    }
    return value;
  }
  if (key.endsWith("_at")) {
    if (typeof value === "number") {
      return DateTime.fromSeconds(value);
    }
  }
  return value;
}

export function luxonReplacer(key: string, value: string): unknown {
  if (lifecycleAtKeys.has(key)) {
    return value;
  }
  if (key.endsWith("_at")) {
    return Math.round(DateTime.fromISO(value).toSeconds());
  }
  return value;
}
