#ifndef PARCEL_API_C_FEATURES_H
#define PARCEL_API_C_FEATURES_H

#include <time.h>

#include "yyjson.h"

// Mirrors the Rust/Kotlin `compute_features` exactly. Reads fields from the
// immutable parsed parcel and pushes feature objects into `out_arr`, which
// must live inside `doc`. `now` is unix epoch seconds (UTC).
void compute_features(yyjson_mut_doc *doc, yyjson_mut_val *out_arr,
                      yyjson_val *parcel, time_t now);

#endif
