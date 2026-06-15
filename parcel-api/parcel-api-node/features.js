// Derived "feature" computation, ported function-for-function from the Go / Rust
// ports. Push order and constants match exactly so the resulting features
// arrays are byte-identical across languages.

const PICKUP_DEADLINE_WINDOW_MS = 48 * 60 * 60 * 1000;
const HEAVY_PARCEL_KG = 5.0;
const VOLUMETRIC_DIVISOR = 5000.0;
const CHECKSUM_MOD = 97;
const HASH_SEED = 0xcbf29ce484222325n;
const HASH_PRIME = 0x100000001b3n;
const U64 = 0xffffffffffffffffn;

const ms = (dateStr) => Date.parse(dateStr);

// Builds a feature with keys inserted in the canonical order
// (type, url, title, description, date); absent keys are skipped.
function feature(type, opts = {}) {
  const f = { type };
  if (opts.url !== undefined) f.url = opts.url;
  if (opts.title !== undefined) f.title = opts.title;
  if (opts.description !== undefined) f.description = opts.description;
  if (opts.date !== undefined) f.date = opts.date;
  return f;
}

export function computeFeatures(p, nowMs) {
  const out = [];
  let f;
  if ((f = customsDocumentsRequired(p))) out.push(f);
  if ((f = heavyParcel(p))) out.push(f);
  if ((f = rateDelivery(p))) out.push(f);
  if ((f = pickupDeadlineSoon(p, nowMs))) out.push(f);
  if ((f = rewardsAvailable(p, nowMs))) out.push(f);
  if ((f = latestEventCause(p))) out.push(f);
  if ((f = greenTransport(p))) out.push(f);
  if ((f = routeSummary(p))) out.push(f);
  if ((f = parcelChecksum(p))) out.push(f);
  if ((f = deliveryProgressBucket(p, nowMs))) out.push(f);
  return out;
}

function customsDocumentsRequired(p) {
  const c = p.customsInformationRequirements;
  if (!c) return null;
  if (!c.documentsRequired || c.informationProvided) return null;
  let pending = 0;
  for (const e of p.events || []) {
    if (e.type === 'customs' && e.cause != null) pending++;
  }
  return feature('CUSTOMS_DOCUMENTS_REQUIRED', {
    title: 'Customs information needed',
    description: `Pending customs events: ${pending}`,
  });
}

function heavyParcel(p) {
  if (typeof p.weightInKg !== 'number') return null;
  const weight = p.weightInKg;
  let volumetric = 0.0;
  const d = p.dimensions;
  if (d) volumetric = (d.lengthInCm * d.widthInCm * d.heightInCm) / VOLUMETRIC_DIVISOR;
  const billable = Math.max(weight, volumetric);
  if (billable <= HEAVY_PARCEL_KG) return null;
  return feature('HEAVY_PARCEL', {
    description:
      `Billable ${billable.toFixed(1)} kg ` +
      `(actual ${weight.toFixed(1)}, volumetric ${volumetric.toFixed(1)})`,
  });
}

function rateDelivery(p) {
  if (p.status !== 'archived' || p.direction !== 'receive') return null;
  let latestDate = null;
  let latestMs = -Infinity;
  for (const e of p.events || []) {
    if (e.type === 'delivered' || e.displayStatus === 'delivered') {
      const m = ms(e.date);
      if (latestDate === null || m > latestMs) {
        latestMs = m;
        latestDate = e.date;
      }
    }
  }
  const opts = { url: `https://posten.no/sporing/${p.parcelNumber}/rate` };
  if (latestDate !== null) opts.date = latestDate;
  return feature('RATE_DELIVERY', opts);
}

function pickupDeadlineSoon(p, nowMs) {
  if (!p.delivery || p.delivery.deadlineDate == null) return null;
  const deadlineStr = p.delivery.deadlineDate;
  const remaining = ms(deadlineStr) - nowMs;
  if (remaining < 0 || remaining >= PICKUP_DEADLINE_WINDOW_MS) return null;
  const hours = Math.trunc(remaining / (60 * 60 * 1000));
  return feature('PICKUP_DEADLINE_SOON', {
    title: `Pick up within ${hours} hours`,
    date: deadlineStr,
  });
}

function rewardsAvailable(p, nowMs) {
  const earnings = p.rewards && p.rewards.rewardsEarnings;
  if (!earnings || earnings.length === 0) return null;
  let activeCoins = 0;
  let bestCoins = -2147483648;
  let bestType = null;
  let haveBest = false;
  for (const e of earnings) {
    if (nowMs < ms(e.validFrom) || nowMs >= ms(e.validTo)) continue;
    activeCoins += e.coins;
    if (e.coins > bestCoins) {
      bestCoins = e.coins;
      bestType = e.type;
      haveBest = true;
    }
  }
  if (activeCoins === 0) return null;
  const opts = { title: `${activeCoins} coins available` };
  if (haveBest) opts.description = `Top reward: ${bestType} (${bestCoins} coins)`;
  return feature('REWARDS_AVAILABLE', opts);
}

function latestEventCause(p) {
  const events = p.events || [];
  if (events.length === 0) return null;
  let idx = 0;
  for (let i = 1; i < events.length; i++) {
    if (ms(events[i].date) > ms(events[idx].date)) idx = i;
  }
  if (events[idx].cause == null) return null;
  return feature('LATEST_EVENT_HAS_CAUSE', {
    description: events[idx].cause,
    date: events[idx].date,
  });
}

function greenTransport(p) {
  if (!p.transport || !p.transport.electric) return null;
  return feature('GREEN_TRANSPORT', {
    description: `Powered by ${p.transport.fuelType}`,
  });
}

function routeSummary(p) {
  const events = p.events || [];
  if (events.length === 0) return null;
  const order = events.map((_, i) => i);
  order.sort((a, b) => ms(events[a].date) - ms(events[b].date));
  const seen = [];
  let hash = HASH_SEED;
  for (const i of order) {
    const e = events[i];
    if (e.city == null) continue;
    const country = e.countryCode != null ? e.countryCode : '??';
    const location = `${e.city},${country}`;
    if (seen.includes(location)) continue;
    // Iterate Unicode code points (matches Rust's `location.chars()` /
    // Go's `range` over a string) so hashes match for non-ASCII cities.
    for (const ch of location) {
      hash = ((hash ^ BigInt(ch.codePointAt(0))) * HASH_PRIME) & U64;
    }
    seen.push(location);
  }
  if (seen.length === 0) return null;
  return feature('ROUTE_SUMMARY', {
    url: `https://posten.no/sporing/${p.parcelNumber}#h${hash.toString(16)}`,
    title: `${seen.length} stops`,
    description: seen.join(' -> '),
  });
}

function parcelChecksum(p) {
  let sum = 0;
  let weight = 1;
  for (const ch of p.parcelNumber) {
    const r = ch.codePointAt(0);
    const digit = r >= 48 && r <= 57 ? r - 48 : r % 10;
    sum += digit * weight;
    weight = weight === 7 ? 1 : weight + 2;
  }
  const remainder = sum % CHECKSUM_MOD;
  if (remainder === 0) return null;
  return feature('PARCEL_CHECKSUM_OK', { description: `checksum=${remainder}` });
}

function deliveryProgressBucket(p, nowMs) {
  const events = p.events || [];
  if (events.length === 0) return null;
  let firstMs = ms(events[0].date);
  for (let i = 1; i < events.length; i++) {
    const m = ms(events[i].date);
    if (m < firstMs) firstMs = m;
  }
  let transitDays = Math.trunc((nowMs - firstMs) / (24 * 60 * 60 * 1000));
  if (transitDays < 0) transitDays = 0;

  let statusWeight = 0;
  switch (p.status) {
    case 'notified': statusWeight = 10; break;
    case 'underway': statusWeight = 30; break;
    case 'collectable': statusWeight = 60; break;
    case 'return_underway': statusWeight = 40; break;
    case 'return_collectable': statusWeight = 70; break;
    case 'archived': statusWeight = 100; break;
    case 'archived_by_user': statusWeight = 90; break;
  }
  let eventScore = events.length * 3;
  if (eventScore > 60) eventScore = 60;
  let timePenalty = transitDays;
  if (timePenalty > 20) timePenalty = 20;
  let score = statusWeight + eventScore - timePenalty;
  if (score < 0) score = 0;
  if (score > 100) score = 100;

  let bucket;
  if (score < 25) bucket = 'early';
  else if (score < 60) bucket = 'mid';
  else if (score < 90) bucket = 'late';
  else bucket = 'complete';

  return feature('DELIVERY_PROGRESS_BUCKET', {
    title: bucket,
    description: `score=${score} transitDays=${transitDays}`,
  });
}
