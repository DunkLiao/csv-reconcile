// Validate syntax without converting the decimal to a JavaScript Number.
// Keep format and resource limits aligned with value_comparator.rs.
export function numericToleranceError(value: string): string | null {
  const format = /^[+-]?(?:(?:[0-9]+|[0-9]{1,3}(?:,[0-9]{3})+)(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$/;
  if (value.length > 4096 || !format.test(value) || /\s/.test(value)) {
    return '請輸入有效的非負數值，最多 4096 字元。';
  }
  const [mantissa, exponent] = value.split(/[eE]/);
  if (mantissa.startsWith('-') && /[1-9]/.test(mantissa)) {
    return '數值誤差容許值不可為負數。';
  }
  if (exponent !== undefined && Math.abs(Number(exponent)) > 4096) {
    return '科學記號指數須介於 -4096 至 4096。';
  }
  return null;
}
