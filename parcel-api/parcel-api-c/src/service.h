#ifndef PARCEL_API_C_SERVICE_H
#define PARCEL_API_C_SERVICE_H

#include <stddef.h>
#include <time.h>

typedef struct service service_t;

service_t *service_load(const char *dir);
void service_free(service_t *svc);
size_t service_count(const service_t *svc);

// Build response JSON for GET /parcel. Caller owns the returned buffer
// and must release it via service_release_response_buffer().
char *service_list_parcels(service_t *svc, size_t *out_len);

// Release a buffer returned by the functions above. yyjson uses its own
// allocator, so libc free() is the wrong call.
void service_release_response_buffer(char *buf);

#endif
