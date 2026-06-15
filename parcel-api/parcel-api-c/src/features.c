#include "features.h"

#include <ctype.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <time.h>

#define PICKUP_DEADLINE_WINDOW_HOURS 48
#define HEAVY_PARCEL_KG 5.0
#define VOLUMETRIC_DIVISOR 5000.0
#define CHECKSUM_MOD 97
#define HASH_SEED 0xcbf29ce484222325ULL
#define HASH_PRIME 0x100000001b3ULL

/* ---------- Read helpers over immutable yyjson values ---------- */

static yyjson_val *get(yyjson_val *obj, const char *key) {
    return yyjson_obj_get(obj, key);
}

static bool get_str(yyjson_val *obj, const char *key, const char **out) {
    yyjson_val *v = yyjson_obj_get(obj, key);
    if (!v || !yyjson_is_str(v)) return false;
    *out = yyjson_get_str(v);
    return true;
}

static bool get_bool(yyjson_val *obj, const char *key, bool *out) {
    yyjson_val *v = yyjson_obj_get(obj, key);
    if (!v || !yyjson_is_bool(v)) return false;
    *out = yyjson_get_bool(v);
    return true;
}

static bool get_int(yyjson_val *obj, const char *key, int64_t *out) {
    yyjson_val *v = yyjson_obj_get(obj, key);
    if (!v || !yyjson_is_int(v)) return false;
    *out = yyjson_get_int(v);
    return true;
}

static bool get_num(yyjson_val *obj, const char *key, double *out) {
    yyjson_val *v = yyjson_obj_get(obj, key);
    if (!v || !yyjson_is_num(v)) return false;
    *out = yyjson_get_num(v);
    return true;
}

static yyjson_val *get_obj(yyjson_val *obj, const char *key) {
    yyjson_val *v = yyjson_obj_get(obj, key);
    return (v && yyjson_is_obj(v)) ? v : NULL;
}

static yyjson_val *get_arr(yyjson_val *obj, const char *key) {
    yyjson_val *v = yyjson_obj_get(obj, key);
    return (v && yyjson_is_arr(v)) ? v : NULL;
}

/* ---------- ISO 8601 parsing ---------- */

// Accepts "YYYY-MM-DDTHH:MM:SSZ" (the format used by parcel-data); falls
// back to ignoring any trailing characters. Returns true on success.
static bool parse_iso8601(const char *s, time_t *out) {
    if (!s) return false;
    struct tm tm = {0};
    int year, mon, day, hh, mm, ss;
    if (sscanf(s, "%4d-%2d-%2dT%2d:%2d:%2d",
               &year, &mon, &day, &hh, &mm, &ss) != 6) return false;
    tm.tm_year = year - 1900;
    tm.tm_mon = mon - 1;
    tm.tm_mday = day;
    tm.tm_hour = hh;
    tm.tm_min = mm;
    tm.tm_sec = ss;
    *out = timegm(&tm);
    return true;
}

/* ---------- Feature object builder ---------- */

// Append a feature to `out_arr`. Each non-NULL string is added as a copy.
// `date_iso` is included verbatim (already in ISO8601) when non-NULL.
static void emit_feature(yyjson_mut_doc *doc, yyjson_mut_val *out_arr,
                         const char *type,
                         const char *url,
                         const char *title,
                         const char *description,
                         const char *date_iso) {
    yyjson_mut_val *f = yyjson_mut_obj(doc);
    yyjson_mut_obj_add_strcpy(doc, f, "type", type);
    if (url) yyjson_mut_obj_add_strcpy(doc, f, "url", url);
    if (title) yyjson_mut_obj_add_strcpy(doc, f, "title", title);
    if (description) yyjson_mut_obj_add_strcpy(doc, f, "description", description);
    if (date_iso) yyjson_mut_obj_add_strcpy(doc, f, "date", date_iso);
    yyjson_mut_arr_append(out_arr, f);
}

/* ---------- Individual feature rules ---------- */

static void feature_customs_documents_required(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                                                yyjson_val *parcel) {
    yyjson_val *customs = get_obj(parcel, "customsInformationRequirements");
    if (!customs) return;
    bool required = false, provided = false;
    get_bool(customs, "documentsRequired", &required);
    get_bool(customs, "informationProvided", &provided);
    if (!required || provided) return;

    yyjson_val *events = get_arr(parcel, "events");
    int pending = 0;
    if (events) {
        size_t idx, max;
        yyjson_val *ev;
        yyjson_arr_foreach(events, idx, max, ev) {
            const char *etype = NULL;
            if (!get_str(ev, "type", &etype)) continue;
            if (strcmp(etype, "customs") != 0) continue;
            yyjson_val *cause = get(ev, "cause");
            if (cause && !yyjson_is_null(cause)) pending++;
        }
    }
    char description[64];
    snprintf(description, sizeof(description), "Pending customs events: %d", pending);
    emit_feature(doc, arr, "CUSTOMS_DOCUMENTS_REQUIRED",
                 NULL, "Customs information needed", description, NULL);
}

static void feature_heavy_parcel(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                                  yyjson_val *parcel) {
    double weight;
    if (!get_num(parcel, "weightInKg", &weight)) return;
    double volumetric = 0.0;
    yyjson_val *dims = get_obj(parcel, "dimensions");
    if (dims) {
        int64_t l = 0, w = 0, h = 0;
        get_int(dims, "lengthInCm", &l);
        get_int(dims, "widthInCm", &w);
        get_int(dims, "heightInCm", &h);
        volumetric = ((double)l * (double)w * (double)h) / VOLUMETRIC_DIVISOR;
    }
    double billable = weight > volumetric ? weight : volumetric;
    if (billable <= HEAVY_PARCEL_KG) return;

    char description[96];
    snprintf(description, sizeof(description),
             "Billable %.1f kg (actual %.1f, volumetric %.1f)",
             billable, weight, volumetric);
    emit_feature(doc, arr, "HEAVY_PARCEL", NULL, NULL, description, NULL);
}

static void feature_rate_delivery(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                                   yyjson_val *parcel) {
    const char *status = NULL;
    const char *direction = NULL;
    if (!get_str(parcel, "status", &status)) return;
    if (!get_str(parcel, "direction", &direction)) return;
    if (strcmp(status, "archived") != 0) return;
    if (strcmp(direction, "receive") != 0) return;

    const char *parcel_number = NULL;
    get_str(parcel, "parcelNumber", &parcel_number);

    // Find latest event with type == "delivered" || displayStatus == "delivered".
    yyjson_val *events = get_arr(parcel, "events");
    const char *latest_date = NULL;
    if (events) {
        size_t idx, max;
        yyjson_val *ev;
        yyjson_arr_foreach(events, idx, max, ev) {
            const char *etype = NULL, *display = NULL, *edate = NULL;
            get_str(ev, "type", &etype);
            get_str(ev, "displayStatus", &display);
            if ((etype && strcmp(etype, "delivered") == 0) ||
                (display && strcmp(display, "delivered") == 0)) {
                if (get_str(ev, "date", &edate)) {
                    if (!latest_date || strcmp(edate, latest_date) > 0) {
                        latest_date = edate;
                    }
                }
            }
        }
    }

    char url[256];
    snprintf(url, sizeof(url), "https://posten.no/sporing/%s/rate",
             parcel_number ? parcel_number : "");
    emit_feature(doc, arr, "RATE_DELIVERY", url, NULL, NULL, latest_date);
}

static void feature_pickup_deadline_soon(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                                          yyjson_val *parcel, time_t now) {
    yyjson_val *delivery = get_obj(parcel, "delivery");
    if (!delivery) return;
    const char *deadline_str = NULL;
    if (!get_str(delivery, "deadlineDate", &deadline_str)) return;
    time_t deadline;
    if (!parse_iso8601(deadline_str, &deadline)) return;
    double remaining_secs = difftime(deadline, now);
    if (remaining_secs < 0) return;
    if (remaining_secs >= PICKUP_DEADLINE_WINDOW_HOURS * 3600) return;
    long remaining_hours = (long)(remaining_secs / 3600);
    char title[64];
    snprintf(title, sizeof(title), "Pick up within %ld hours", remaining_hours);
    emit_feature(doc, arr, "PICKUP_DEADLINE_SOON",
                 NULL, title, NULL, deadline_str);
}

static void feature_rewards_available(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                                       yyjson_val *parcel, time_t now) {
    yyjson_val *rewards = get_obj(parcel, "rewards");
    if (!rewards) return;
    yyjson_val *earnings = get_arr(rewards, "rewardsEarnings");
    if (!earnings) return;

    int64_t active_coins = 0;
    int64_t best_coins = INT64_MIN;
    const char *best_type = NULL;
    size_t idx, max;
    yyjson_val *e;
    yyjson_arr_foreach(earnings, idx, max, e) {
        const char *from = NULL, *to = NULL;
        if (!get_str(e, "validFrom", &from) || !get_str(e, "validTo", &to)) continue;
        time_t tfrom, tto;
        if (!parse_iso8601(from, &tfrom) || !parse_iso8601(to, &tto)) continue;
        if (difftime(now, tfrom) < 0) continue;
        if (difftime(now, tto) >= 0) continue;
        int64_t coins = 0;
        get_int(e, "coins", &coins);
        active_coins += coins;
        if (coins > best_coins) {
            best_coins = coins;
            get_str(e, "type", &best_type);
        }
    }
    if (active_coins == 0) return;

    char title[64];
    snprintf(title, sizeof(title), "%lld coins available", (long long)active_coins);
    const char *description = NULL;
    char desc_buf[96];
    if (best_type) {
        snprintf(desc_buf, sizeof(desc_buf),
                 "Top reward: %s (%lld coins)", best_type, (long long)best_coins);
        description = desc_buf;
    }
    emit_feature(doc, arr, "REWARDS_AVAILABLE", NULL, title, description, NULL);
}

static void feature_latest_event_cause(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                                        yyjson_val *parcel) {
    yyjson_val *events = get_arr(parcel, "events");
    if (!events) return;
    const char *latest_date = NULL;
    const char *latest_cause = NULL;
    size_t idx, max;
    yyjson_val *ev;
    yyjson_arr_foreach(events, idx, max, ev) {
        const char *edate = NULL;
        if (!get_str(ev, "date", &edate)) continue;
        if (latest_date && strcmp(edate, latest_date) <= 0) continue;
        latest_date = edate;
        yyjson_val *cause = get(ev, "cause");
        latest_cause = (cause && yyjson_is_str(cause)) ? yyjson_get_str(cause) : NULL;
    }
    if (!latest_cause) return;
    emit_feature(doc, arr, "LATEST_EVENT_HAS_CAUSE",
                 NULL, NULL, latest_cause, latest_date);
}

static void feature_green_transport(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                                     yyjson_val *parcel) {
    yyjson_val *transport = get_obj(parcel, "transport");
    if (!transport) return;
    bool electric = false;
    if (!get_bool(transport, "electric", &electric) || !electric) return;
    const char *fuel = NULL;
    get_str(transport, "fuelType", &fuel);
    char description[64];
    snprintf(description, sizeof(description), "Powered by %s",
             fuel ? fuel : "");
    emit_feature(doc, arr, "GREEN_TRANSPORT", NULL, NULL, description, NULL);
}

/* ---- route_summary helpers ---- */

// Indices into an events array sorted by ISO 8601 date string (lexicographic
// ordering matches chronological for the Zulu-suffix format the input uses).
typedef struct { size_t idx; const char *date; } event_ref_t;
static int compare_event_refs(const void *a, const void *b) {
    return strcmp(((const event_ref_t *)a)->date,
                  ((const event_ref_t *)b)->date);
}

static void feature_route_summary(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                                   yyjson_val *parcel) {
    yyjson_val *events = get_arr(parcel, "events");
    if (!events) return;
    size_t n = yyjson_arr_size(events);
    if (n == 0) return;

    event_ref_t *refs = malloc(n * sizeof(*refs));
    size_t nrefs = 0;
    size_t idx, max;
    yyjson_val *ev;
    yyjson_arr_foreach(events, idx, max, ev) {
        const char *edate = NULL;
        if (!get_str(ev, "date", &edate)) continue;
        refs[nrefs].idx = idx;
        refs[nrefs].date = edate;
        nrefs++;
    }
    qsort(refs, nrefs, sizeof(*refs), compare_event_refs);

    // Build route by dedupe-and-append on "city,country" pairs in time order.
    char **seen = malloc(nrefs * sizeof(char *));
    size_t seen_n = 0;
    uint64_t hash = HASH_SEED;
    for (size_t k = 0; k < nrefs; k++) {
        yyjson_val *e = yyjson_arr_get(events, refs[k].idx);
        const char *city = NULL;
        const char *country = "??";
        if (!get_str(e, "city", &city)) continue;
        get_str(e, "countryCode", &country);
        char location[128];
        snprintf(location, sizeof(location), "%s,%s", city, country);

        bool dup = false;
        for (size_t s = 0; s < seen_n; s++) {
            if (strcmp(seen[s], location) == 0) { dup = true; break; }
        }
        if (dup) continue;

        // Mix in one Unicode code point at a time (UTF-8 decode), matching
        // Rust's `for ch in location.chars()` so the hash is identical for
        // city names with non-ASCII characters (e.g. Tromsø).
        const unsigned char *p = (const unsigned char *)location;
        while (*p) {
            uint32_t cp;
            unsigned char c = *p++;
            if (c < 0x80) {
                cp = c;
            } else if ((c & 0xE0) == 0xC0 && (p[0] & 0xC0) == 0x80) {
                cp = ((uint32_t)(c & 0x1F) << 6) | (uint32_t)(p[0] & 0x3F);
                p += 1;
            } else if ((c & 0xF0) == 0xE0 &&
                       (p[0] & 0xC0) == 0x80 && (p[1] & 0xC0) == 0x80) {
                cp = ((uint32_t)(c & 0x0F) << 12) |
                     ((uint32_t)(p[0] & 0x3F) << 6) |
                     (uint32_t)(p[1] & 0x3F);
                p += 2;
            } else if ((c & 0xF8) == 0xF0 &&
                       (p[0] & 0xC0) == 0x80 && (p[1] & 0xC0) == 0x80 &&
                       (p[2] & 0xC0) == 0x80) {
                cp = ((uint32_t)(c & 0x07) << 18) |
                     ((uint32_t)(p[0] & 0x3F) << 12) |
                     ((uint32_t)(p[1] & 0x3F) << 6) |
                     (uint32_t)(p[2] & 0x3F);
                p += 3;
            } else {
                // Malformed sequence — match Rust's strict UTF-8 by skipping.
                continue;
            }
            hash = (hash ^ (uint64_t)cp) * HASH_PRIME;
        }
        seen[seen_n++] = strdup(location);
    }
    free(refs);

    if (seen_n == 0) {
        free(seen);
        return;
    }

    // Compute total length for the joined route string and url.
    size_t route_len = 0;
    for (size_t s = 0; s < seen_n; s++) {
        route_len += strlen(seen[s]);
        if (s + 1 < seen_n) route_len += 4; // " -> "
    }
    char *route = malloc(route_len + 1);
    size_t off = 0;
    for (size_t s = 0; s < seen_n; s++) {
        size_t l = strlen(seen[s]);
        memcpy(route + off, seen[s], l);
        off += l;
        if (s + 1 < seen_n) {
            memcpy(route + off, " -> ", 4);
            off += 4;
        }
    }
    route[off] = '\0';

    const char *parcel_number = NULL;
    get_str(parcel, "parcelNumber", &parcel_number);
    char title[32];
    snprintf(title, sizeof(title), "%zu stops", seen_n);
    char url[256];
    snprintf(url, sizeof(url),
             "https://posten.no/sporing/%s#h%llx",
             parcel_number ? parcel_number : "",
             (unsigned long long)hash);
    emit_feature(doc, arr, "ROUTE_SUMMARY", url, title, route, NULL);

    free(route);
    for (size_t s = 0; s < seen_n; s++) free(seen[s]);
    free(seen);
}

static void feature_parcel_checksum(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                                     yyjson_val *parcel) {
    const char *number = NULL;
    if (!get_str(parcel, "parcelNumber", &number)) return;
    int sum = 0, weight = 1;
    for (const char *p = number; *p; p++) {
        int digit;
        if (isdigit((unsigned char)*p)) digit = *p - '0';
        else digit = (int)(unsigned char)*p % 10;
        sum += digit * weight;
        weight = (weight == 7) ? 1 : weight + 2;
    }
    int remainder = sum % CHECKSUM_MOD;
    if (remainder == 0) return;
    char description[32];
    snprintf(description, sizeof(description), "checksum=%d", remainder);
    emit_feature(doc, arr, "PARCEL_CHECKSUM_OK", NULL, NULL, description, NULL);
}

static void feature_delivery_progress_bucket(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                                              yyjson_val *parcel, time_t now) {
    yyjson_val *events = get_arr(parcel, "events");
    if (!events) return;
    size_t n = yyjson_arr_size(events);
    if (n == 0) return;

    const char *min_date = NULL;
    size_t idx, max;
    yyjson_val *ev;
    yyjson_arr_foreach(events, idx, max, ev) {
        const char *edate = NULL;
        if (!get_str(ev, "date", &edate)) continue;
        if (!min_date || strcmp(edate, min_date) < 0) min_date = edate;
    }
    if (!min_date) return;
    time_t tfirst;
    if (!parse_iso8601(min_date, &tfirst)) return;
    long transit_days = (long)(difftime(now, tfirst) / 86400.0);
    if (transit_days < 0) transit_days = 0;

    const char *status = NULL;
    get_str(parcel, "status", &status);
    int status_weight = 0;
    if (status) {
        if      (strcmp(status, "notified") == 0) status_weight = 10;
        else if (strcmp(status, "underway") == 0) status_weight = 30;
        else if (strcmp(status, "collectable") == 0) status_weight = 60;
        else if (strcmp(status, "return_underway") == 0) status_weight = 40;
        else if (strcmp(status, "return_collectable") == 0) status_weight = 70;
        else if (strcmp(status, "archived") == 0) status_weight = 100;
        else if (strcmp(status, "archived_by_user") == 0) status_weight = 90;
    }
    int event_score = (int)n * 3;
    if (event_score > 60) event_score = 60;
    int time_penalty = transit_days > 20 ? 20 : (int)transit_days;
    int score = status_weight + event_score - time_penalty;
    if (score < 0) score = 0;
    if (score > 100) score = 100;

    const char *bucket;
    if (score < 25) bucket = "early";
    else if (score < 60) bucket = "mid";
    else if (score < 90) bucket = "late";
    else bucket = "complete";

    char description[64];
    snprintf(description, sizeof(description),
             "score=%d transitDays=%ld", score, transit_days);
    emit_feature(doc, arr, "DELIVERY_PROGRESS_BUCKET",
                 NULL, bucket, description, NULL);
}

/* ---------- Public entry ---------- */

void compute_features(yyjson_mut_doc *doc, yyjson_mut_val *arr,
                      yyjson_val *parcel, time_t now) {
    feature_customs_documents_required(doc, arr, parcel);
    feature_heavy_parcel(doc, arr, parcel);
    feature_rate_delivery(doc, arr, parcel);
    feature_pickup_deadline_soon(doc, arr, parcel, now);
    feature_rewards_available(doc, arr, parcel, now);
    feature_latest_event_cause(doc, arr, parcel);
    feature_green_transport(doc, arr, parcel);
    feature_route_summary(doc, arr, parcel);
    feature_parcel_checksum(doc, arr, parcel);
    feature_delivery_progress_bucket(doc, arr, parcel, now);
}
