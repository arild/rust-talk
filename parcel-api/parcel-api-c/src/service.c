#include "service.h"

#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>

#include "features.h"
#include "yyjson.h"

typedef struct {
    char *number;
    char *json;
    size_t json_len;
} parcel_raw_t;

struct service {
    parcel_raw_t *parcels;
    size_t count;
};

static int compare_parcel_raw(const void *a, const void *b) {
    return strcmp(((const parcel_raw_t *)a)->number,
                  ((const parcel_raw_t *)b)->number);
}

static char *read_file(const char *path, size_t *out_len) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    if (fseek(f, 0, SEEK_END) != 0) { fclose(f); return NULL; }
    long size = ftell(f);
    if (size < 0) { fclose(f); return NULL; }
    rewind(f);
    char *buf = malloc((size_t)size + 1);
    if (!buf) { fclose(f); return NULL; }
    size_t n = fread(buf, 1, (size_t)size, f);
    fclose(f);
    if (n != (size_t)size) { free(buf); return NULL; }
    buf[size] = '\0';
    *out_len = (size_t)size;
    return buf;
}

service_t *service_load(const char *dir) {
    DIR *d = opendir(dir);
    if (!d) {
        fprintf(stderr, "service_load: opendir %s failed: %s\n", dir, strerror(errno));
        return NULL;
    }

    service_t *svc = calloc(1, sizeof(*svc));
    if (!svc) { closedir(d); return NULL; }
    size_t cap = 16;
    svc->parcels = calloc(cap, sizeof(parcel_raw_t));
    if (!svc->parcels) { closedir(d); free(svc); return NULL; }

    struct dirent *entry;
    while ((entry = readdir(d)) != NULL) {
        const char *name = entry->d_name;
        size_t name_len = strlen(name);
        if (name_len < 6) continue;
        if (strcmp(name + name_len - 5, ".json") != 0) continue;

        if (svc->count >= cap) {
            cap *= 2;
            parcel_raw_t *grown = realloc(svc->parcels, cap * sizeof(parcel_raw_t));
            if (!grown) break;
            svc->parcels = grown;
        }

        char path[1024];
        snprintf(path, sizeof(path), "%s/%s", dir, name);
        size_t json_len = 0;
        char *json = read_file(path, &json_len);
        if (!json) {
            fprintf(stderr, "skip %s: read failed\n", path);
            continue;
        }

        char *number = strndup(name, name_len - 5);
        svc->parcels[svc->count].number = number;
        svc->parcels[svc->count].json = json;
        svc->parcels[svc->count].json_len = json_len;
        svc->count++;
    }
    closedir(d);

    qsort(svc->parcels, svc->count, sizeof(parcel_raw_t), compare_parcel_raw);
    return svc;
}

void service_free(service_t *svc) {
    if (!svc) return;
    for (size_t i = 0; i < svc->count; i++) {
        free(svc->parcels[i].number);
        free(svc->parcels[i].json);
    }
    free(svc->parcels);
    free(svc);
}

size_t service_count(const service_t *svc) {
    return svc ? svc->count : 0;
}

// Fisher-Yates shuffle of an index array. Uses rand() because we don't need
// crypto quality; matches the spirit of Kotlin's Collections.shuffle().
static void shuffle_indices(size_t *idx, size_t n) {
    for (size_t i = n; i > 1; i--) {
        size_t j = (size_t)rand() % i;
        size_t tmp = idx[i - 1];
        idx[i - 1] = idx[j];
        idx[j] = tmp;
    }
}

// Walks the mutable JSON tree and removes any key whose value is null.
// Mirrors `#[serde(skip_serializing_if = "Option::is_none")]` on every
// Option field in the Rust DTOs — the input JSON puts explicit nulls in
// those slots, which the Rust serializer drops on the way out.
static void strip_nulls(yyjson_mut_val *val) {
    if (val == NULL) return;
    if (yyjson_mut_is_obj(val)) {
        // yyjson exposes a mutable iterator that supports removal in-place.
        yyjson_mut_obj_iter iter;
        yyjson_mut_obj_iter_init(val, &iter);
        yyjson_mut_val *key;
        while ((key = yyjson_mut_obj_iter_next(&iter)) != NULL) {
            yyjson_mut_val *child = yyjson_mut_obj_iter_get_val(key);
            if (yyjson_mut_is_null(child)) {
                yyjson_mut_obj_iter_remove(&iter);
            } else {
                strip_nulls(child);
            }
        }
    } else if (yyjson_mut_is_arr(val)) {
        size_t idx, max;
        yyjson_mut_val *item;
        yyjson_mut_arr_foreach(val, idx, max, item) {
            strip_nulls(item);
        }
    }
}

// Build a yyjson_mut_val deep-copy of `parcel` inside `out_doc`, then attach
// a freshly computed features array under the "features" key.
static yyjson_mut_val *materialize_parcel(yyjson_mut_doc *out_doc,
                                          yyjson_val *parcel,
                                          time_t now) {
    yyjson_mut_val *mut_parcel = yyjson_val_mut_copy(out_doc, parcel);
    if (!mut_parcel) return NULL;

    yyjson_mut_val *features = yyjson_mut_arr(out_doc);
    compute_features(out_doc, features, parcel, now);

    // Replace the input's "features" value in-place so the key keeps its
    // original position. The Rust serializer emits fields in struct order,
    // which matches the order in the input JSON files — keeping the key
    // here yields byte-identical output.
    yyjson_mut_val *key = yyjson_mut_strn(out_doc, "features", 8);
    if (!yyjson_mut_obj_replace(mut_parcel, key, features)) {
        yyjson_mut_obj_add_val(out_doc, mut_parcel, "features", features);
    }

    // Drop the explicit-null fields the input carries so the wire format
    // matches the other variants.
    strip_nulls(mut_parcel);
    return mut_parcel;
}

char *service_list_parcels(service_t *svc, size_t *out_len) {
    if (!svc) return NULL;
    time_t now = time(NULL);

    yyjson_mut_doc *out_doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *parcels_arr = yyjson_mut_arr(out_doc);
    yyjson_mut_doc_set_root(out_doc, parcels_arr);

    size_t *order = malloc(svc->count * sizeof(size_t));
    for (size_t i = 0; i < svc->count; i++) order[i] = i;
    shuffle_indices(order, svc->count);

    for (size_t k = 0; k < svc->count; k++) {
        const parcel_raw_t *raw = &svc->parcels[order[k]];
        yyjson_doc *parsed = yyjson_read(raw->json, raw->json_len, 0);
        if (!parsed) continue;
        yyjson_val *parcel_val = yyjson_doc_get_root(parsed);
        yyjson_mut_val *mut_parcel = materialize_parcel(out_doc, parcel_val, now);
        if (mut_parcel) yyjson_mut_arr_append(parcels_arr, mut_parcel);
        yyjson_doc_free(parsed);
    }
    free(order);

    size_t len = 0;
    char *json = yyjson_mut_write(out_doc, 0, &len);
    yyjson_mut_doc_free(out_doc);
    if (out_len) *out_len = len;
    return json;
}

void service_release_response_buffer(char *buf) {
    // yyjson_mut_write uses the default allocator, whose free is libc free.
    free(buf);
}
