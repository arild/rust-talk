/*
 * parcel-api: C reimplementation for the rust-talk benchmark.
 *
 * Single-threaded h2o event loop, yyjson for parse + serialize. Endpoints
 * mirror the rust / spring-boot / quarkus variants verbatim:
 *   GET  /parcel-api/check/status
 *   POST /parcel-api/v1/parcel
 */

#include <arpa/inet.h>
#include <errno.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#include "h2o.h"

#include "service.h"

#define DEFAULT_PORT 8080
#define DEFAULT_DATA_DIR "/parcel-data"
#define CONTEXT_PATH "/parcel-api"
#define CHECK_PATH "/parcel-api/check"
#define CHECK_STATUS_PATH "/parcel-api/check/status"
#define PARCEL_PATH_PREFIX "/parcel-api/v1/parcel"

static h2o_globalconf_t config;
static h2o_context_t ctx;
static h2o_accept_ctx_t accept_ctx;
static service_t *svc;

static const char *status_body =
    "\xf0\x9f\x91\x8b parcel-api is on air"; // UTF-8 wave hand + text

static void send_json(h2o_req_t *req, int status, const char *reason,
                      char *json, size_t json_len) {
    // h2o_strdup copies into the request pool; we can then release the
    // original yyjson buffer immediately.
    h2o_iovec_t body = h2o_strdup(&req->pool, json, json_len);
    service_release_response_buffer(json);

    req->res.status = status;
    req->res.reason = reason;
    req->res.content_length = body.len;
    h2o_add_header(&req->pool, &req->res.headers, H2O_TOKEN_CONTENT_TYPE, NULL,
                   H2O_STRLIT("application/json"));

    static h2o_generator_t generator = {NULL, NULL};
    h2o_start_response(req, &generator);
    h2o_send(req, &body, 1, H2O_SEND_STATE_FINAL);
}

// Emits the same {"message":"..."} shape as the Rust ErrorResponse. The JVM
// variants emit Spring/Quarkus defaults that differ in detail — close enough
// for this perf comparison.
static void send_problem(h2o_req_t *req, int status, const char *reason,
                          const char *message) {
    char body[512];
    // Naive JSON-escape: handle the few characters that can appear in our
    // messages (parcel numbers are alphanumeric; the only risk is " in
    // user-supplied text, which we still defend against).
    char escaped[384];
    size_t e = 0;
    for (const char *p = message; *p && e + 2 < sizeof(escaped); p++) {
        if (*p == '"' || *p == '\\') escaped[e++] = '\\';
        escaped[e++] = *p;
    }
    escaped[e] = '\0';
    int n = snprintf(body, sizeof(body), "{\"message\":\"%s\"}", escaped);
    if (n < 0) n = 0;
    h2o_iovec_t buf = h2o_strdup(&req->pool, body, (size_t)n);

    req->res.status = status;
    req->res.reason = reason;
    req->res.content_length = buf.len;
    h2o_add_header(&req->pool, &req->res.headers, H2O_TOKEN_CONTENT_TYPE, NULL,
                   H2O_STRLIT("application/json"));

    static h2o_generator_t generator = {NULL, NULL};
    h2o_start_response(req, &generator);
    h2o_send(req, &buf, 1, H2O_SEND_STATE_FINAL);
}

/* ---------- Route handlers ---------- */

static int send_plain(h2o_req_t *req, const char *body_text, size_t body_len) {
    h2o_iovec_t body = h2o_strdup(&req->pool, body_text, body_len);
    req->res.status = 200;
    req->res.reason = "OK";
    req->res.content_length = body.len;
    h2o_add_header(&req->pool, &req->res.headers, H2O_TOKEN_CONTENT_TYPE, NULL,
                   H2O_STRLIT("text/plain; charset=UTF-8"));
    static h2o_generator_t generator = {NULL, NULL};
    h2o_start_response(req, &generator);
    h2o_send(req, &body, 1, H2O_SEND_STATE_FINAL);
    return 0;
}

static int on_check_status(h2o_handler_t *self, h2o_req_t *req) {
    (void)self;
    if (!h2o_memis(req->method.base, req->method.len, H2O_STRLIT("GET")))
        return -1;
    return send_plain(req, status_body, strlen(status_body));
}

// Mirrors the Rust /check endpoint: banner + cheap memory readout. Reads
// /proc/self/statm directly so it stays a few syscalls — the goal is for
// container probes to be virtually free.
static int on_check(h2o_handler_t *self, h2o_req_t *req) {
    (void)self;
    if (!h2o_memis(req->method.base, req->method.len, H2O_STRLIT("GET")))
        return -1;

    char mem_line[128];
    snprintf(mem_line, sizeof(mem_line), "memory stats unavailable on this platform");
    FILE *fp = fopen("/proc/self/statm", "r");
    if (fp) {
        long total_pages = 0, rss_pages = 0;
        if (fscanf(fp, "%ld %ld", &total_pages, &rss_pages) == 2) {
            long page_kib = sysconf(_SC_PAGESIZE) / 1024;
            long total_mib = total_pages * page_kib / 1024;
            long rss_mib = rss_pages * page_kib / 1024;
            snprintf(mem_line, sizeof(mem_line),
                     "total: %ldmb, resident: %ldmb", total_mib, rss_mib);
        }
        fclose(fp);
    }
    char body[512];
    int n = snprintf(body, sizeof(body),
                     "parcel-api\n\nMemory:\n%s\n\nVersion:\ndev\n", mem_line);
    if (n < 0) n = 0;
    return send_plain(req, body, (size_t)n);
}

// Dispatches POST /parcel-api/v1/parcel.
static int on_parcel(h2o_handler_t *self, h2o_req_t *req) {
    (void)self;
    // Slice off the query string for path matching.
    size_t path_len = req->path.len;
    const char *qmark = memchr(req->path.base, '?', path_len);
    if (qmark) path_len = (size_t)(qmark - req->path.base);

    bool is_post = h2o_memis(req->method.base, req->method.len, H2O_STRLIT("POST"));
    bool exact   = h2o_memis(req->path.base, path_len, H2O_STRLIT(PARCEL_PATH_PREFIX));

    if (is_post && exact) {
        size_t len = 0;
        char *json = service_list_parcels(svc, &len);
        if (!json) {
            send_problem(req, 500, "Internal Server Error", "list_parcels failed");
            return 0;
        }
        send_json(req, 200, "OK", json, len);
        return 0;
    }

    return -1; // let h2o produce a default 405 / 404
}

/* ---------- Boilerplate: handler registration + accept loop ---------- */

static void register_path(h2o_hostconf_t *hostconf, const char *path,
                          int (*on_req)(h2o_handler_t *, h2o_req_t *)) {
    h2o_pathconf_t *pathconf = h2o_config_register_path(hostconf, path, 0);
    h2o_handler_t *handler = h2o_create_handler(pathconf, sizeof(*handler));
    handler->on_req = on_req;
}

static void on_accept(h2o_socket_t *listener, const char *err) {
    if (err != NULL) return;
    h2o_socket_t *sock = h2o_evloop_socket_accept(listener);
    if (sock == NULL) return;
    h2o_accept(&accept_ctx, sock);
}

static int create_listener(uint16_t port) {
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons(port);

    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd == -1) return -1;
    int one = 1;
    if (setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &one, sizeof(one)) != 0) return -1;
    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) return -1;
    if (listen(fd, SOMAXCONN) != 0) return -1;

    h2o_socket_t *sock = h2o_evloop_socket_create(ctx.loop, fd, H2O_SOCKET_FLAG_DONT_READ);
    h2o_socket_read_start(sock, on_accept);
    return 0;
}

int main(int argc, char **argv) {
    (void)argc; (void)argv;
    signal(SIGPIPE, SIG_IGN);

    const char *data_dir = getenv("PARCEL_DATA_DIR");
    if (!data_dir || !*data_dir) data_dir = DEFAULT_DATA_DIR;

    fprintf(stderr, "loading parcel data from %s\n", data_dir);
    svc = service_load(data_dir);
    if (!svc) {
        fprintf(stderr, "service_load failed for %s\n", data_dir);
        return 1;
    }
    fprintf(stderr, "loaded %zu parcels\n", service_count(svc));

    h2o_config_init(&config);
    h2o_hostconf_t *hostconf = h2o_config_register_host(
        &config, h2o_iovec_init(H2O_STRLIT("default")), 65535);

    // Order matters: h2o picks the first matching prefix. Register the more
    // specific path first.
    register_path(hostconf, CHECK_STATUS_PATH, on_check_status);
    register_path(hostconf, CHECK_PATH, on_check);
    register_path(hostconf, PARCEL_PATH_PREFIX, on_parcel);

    h2o_context_init(&ctx, h2o_evloop_create(), &config);
    accept_ctx.ctx = &ctx;
    accept_ctx.hosts = config.hosts;

    if (create_listener(DEFAULT_PORT) != 0) {
        fprintf(stderr, "listen on :%d failed: %s\n", DEFAULT_PORT, strerror(errno));
        return 1;
    }
    fprintf(stderr, "parcel-api-c listening on :%d\n", DEFAULT_PORT);

    while (h2o_evloop_run(ctx.loop, INT32_MAX) == 0) { /* loop */ }

    service_free(svc);
    return 0;
}
