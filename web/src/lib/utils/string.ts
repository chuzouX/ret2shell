/**
 * Calculate the display width of a string, counting CJK (Chinese, Japanese,
 * Korean) and other double-width characters as 2, and all others as 1.
 * Used for typewriter animation max-width calculation.
 */
export function strDisplayWidth(str: string): number {
  let width = 0;
  for (const char of str) {
    const code = char.codePointAt(0) ?? 0;
    if (
      // Hangul Jamo
      (code >= 0x1100 && code <= 0x115f) ||
      // CJK Radicals / Kangxi
      (code >= 0x2e80 && code <= 0x303e) ||
      // Kana / CJK punctuation
      (code >= 0x3040 && code <= 0x33bf) ||
      // CJK Extension A
      (code >= 0x3400 && code <= 0x4dbf) ||
      // CJK Unified Ideographs
      (code >= 0x4e00 && code <= 0xa4cf) ||
      // Hangul Syllables
      (code >= 0xac00 && code <= 0xd7a3) ||
      // CJK Compatibility Ideographs
      (code >= 0xf900 && code <= 0xfaff) ||
      // CJK Compatibility Forms
      (code >= 0xfe30 && code <= 0xfe6f) ||
      // Fullwidth Forms
      (code >= 0xff01 && code <= 0xff60) ||
      // Fullwidth Signs
      (code >= 0xffe0 && code <= 0xffe6) ||
      // CJK Extension B+
      (code >= 0x20000 && code <= 0x2fffd) ||
      (code >= 0x30000 && code <= 0x3fffd)
    ) {
      width += 2;
    } else {
      width += 1;
    }
  }
  return width;
}
