// Custom JSON serialization tuned for byte-for-byte parity with the Go / Rust /
// JVM parcel-api ports:
//   - object keys emit in insertion order (the input fixtures are already in the
//     canonical field order, and computeFeatures replaces `features` in place);
//   - null / undefined properties are omitted (matches Jackson `non_null`,
//     serde `skip_serializing_if = Option::is_none`, and Go `omitempty`);
//   - Float values keep their decimal point (`5` -> `5.0`), matching serde_json
//     and the Go `Float` type — plain JSON.stringify would drop it;
//   - strings go through JSON.stringify, which (like serde_json and Go's encoder
//     with HTML escaping off) leaves &, <, > intact.
// Dates are already canonical whole-second `...Z` strings in the input, so they
// pass through unchanged.

export class Float {
  constructor(value) {
    this.value = value;
  }
}

function formatFloat(n) {
  const s = String(n);
  return /[.eE]/.test(s) ? s : s + '.0';
}

export function serialize(value) {
  if (value instanceof Float) return formatFloat(value.value);
  const type = typeof value;
  if (type === 'string') return JSON.stringify(value);
  if (type === 'number' || type === 'boolean') return JSON.stringify(value);
  if (value === null || value === undefined) return 'null';
  if (Array.isArray(value)) {
    let out = '[';
    for (let i = 0; i < value.length; i++) {
      if (i > 0) out += ',';
      out += serialize(value[i]);
    }
    return out + ']';
  }
  let out = '{';
  let first = true;
  for (const key of Object.keys(value)) {
    const v = value[key];
    if (v === null || v === undefined) continue;
    if (!first) out += ',';
    first = false;
    out += JSON.stringify(key) + ':' + serialize(v);
  }
  return out + '}';
}
