import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { computeFeatures } from './features.js';
import { serialize, Float } from './serialize.js';

// Stores parcel JSON as raw strings in memory; re-parses on every request.
// Mirrors the other ports — putting the JSON parse + serialize cost on the
// request path so the bench measures the work a real adapter does when decoding
// data from a downstream.
export class StubParcelService {
  constructor(dir) {
    const files = readdirSync(dir)
      .filter((f) => f.endsWith('.json'))
      .sort();
    this.parcelJsons = files.map((f) => readFileSync(join(dir, f), 'utf8'));
  }

  get count() {
    return this.parcelJsons.length;
  }

  listParcels() {
    const nowMs = Date.now();
    const order = this.parcelJsons.map((_, i) => i);
    shuffle(order);

    let out = '[';
    for (let k = 0; k < order.length; k++) {
      if (k > 0) out += ',';
      const parcel = JSON.parse(this.parcelJsons[order[k]]);
      // Features are read from the raw numeric weightInKg, so compute before wrapping.
      parcel.features = computeFeatures(parcel, nowMs);
      if (typeof parcel.weightInKg === 'number') {
        parcel.weightInKg = new Float(parcel.weightInKg);
      }
      out += serialize(parcel);
    }
    return out + ']';
  }
}

function shuffle(arr) {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
}
