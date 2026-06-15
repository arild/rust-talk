#!/usr/bin/env bash
#
# bench.sh — measure cold-start time and container RSS for the three
# parcel-api variants.
#
# Usage:
#   ./bench.sh <variant> [iterations]
#
#   variant     spring-boot | quarkus | rust | all
#   iterations  cold-start runs per variant (default: 10)
#
# Requires docker, curl, python3 (for sub-second timing — macOS `date` lacks %N).
# Assumes the Docker images are already built and tagged:
#   parcel-api-spring-boot, parcel-api-quarkus-jvm, parcel-api-quarkus-native,
#   parcel-api-rust, parcel-api-c, parcel-api-go, parcel-api-node
# With `all`, missing variants are skipped with a warning (lets you run before
# all three ports exist).
#
# Set BUILD=1 to clean-build each variant (gradle clean + bootJar / quarkus
# build, then `docker build --no-cache`) before benching. The wall-clock build
# time is shown as a "Build (s)" column in the results table.

set -euo pipefail

[ $# -lt 1 ] && { sed -n '3,/^set -/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//'; exit 1; }

VARIANT="$1"
ITERS="${2:-10}"

CONTAINER=parcel-api-srv
HOST_PORT=8080
BASE_URL="http://localhost:${HOST_PORT}/parcel-api"
READY_URL="${BASE_URL}/check/status"
# Load against the real list-parcels endpoint. Each variant's stub is in-process
# so all three do equivalent work: parse the empty request, serialize ~50 parcels
# (~200 KB JSON). That stresses the runtime's JSON path, which is the comparison
# we want — unlike branded-tracking, the POST path has no external dependency.
# Load runs from a vegeta container on the same bridge; uses container DNS.
LOAD_URL="http://${CONTAINER}:8080/parcel-api/v1/parcel"
NETWORK="bench-net"
VEGETA_LOADER="vegeta-loader"
VEGETA_IMAGE="${VEGETA_IMAGE:-peterevans/vegeta:latest}"
WARMUP_SECONDS=5      # phase 1: 1 connection, sequential
RAMP_SECONDS=15       # phase 2: half MAX_PARALLEL
STEADY_SECONDS=30     # phase 3: MAX_PARALLEL connections
MAX_PARALLEL=200
CPU_LIMIT="${CPU_LIMIT:-3}"   # --cpus passed to docker run (overridable via env)
MEM_LIMIT="${MEM_LIMIT:-}"    # --memory passed to docker run (empty = unlimited; e.g. 1g).
                              # Set this to mirror a k8s pod limit so every GC sizes
                              # its heap to the same cgroup budget (fair RSS comparison).

# Each run lands in its own bench-results/YYYY-MM-DD-N directory; N starts at 1
# and increments per run on the same calendar day so successive runs never
# overwrite each other.
if [ -z "${BENCH_DIR:-}" ]; then
  _date_prefix="bench-results/$(date +%Y-%m-%d)"
  _n=1
  while [ -e "${_date_prefix}-${_n}" ]; do _n=$(( _n + 1 )); done
  BENCH_DIR="${_date_prefix}-${_n}"
fi
BUILD="${BUILD:-0}"           # set BUILD=1 to clean-build each unique image + measure time

# Variant catalog. JVM apps run under the two best-performing GCs in this
# CPU-capped throughput shootout: G1 (default) and ParallelGC. Concurrent
# collectors (ZGC, Shenandoah) ranked far behind in earlier runs because they
# spend ~1 core on concurrent GC threads under the 3-core cap.
# Variants share images: spring-boot-* all use parcel-api-spring-boot, etc.
ALL_VARIANTS=(
  spring-boot-g1 spring-boot-parallel
  quarkus-jvm-g1 quarkus-jvm-parallel
  quarkus-native
  quarkus-native-g1
  rust
  c
  go
  node
)

cleanup() {
  [ -n "${SAMPLER_PID:-}" ] && kill "$SAMPLER_PID" 2>/dev/null && wait "$SAMPLER_PID" 2>/dev/null
  SAMPLER_PID=
  docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
}
final_cleanup() {
  cleanup
  docker rm -f "$VEGETA_LOADER" >/dev/null 2>&1 || true
  docker network rm "$NETWORK" >/dev/null 2>&1 || true
}
trap final_cleanup EXIT

# Spin up the bridge + a long-lived vegeta container with $BENCH_DIR mounted at
# /bench-results so binary outputs land directly in the host's results dir.
# Targets file is written once inside the loader.
setup_loader() {
  local bench_dir_abs=$1
  docker network inspect "$NETWORK" >/dev/null 2>&1 || \
    docker network create "$NETWORK" >/dev/null
  docker rm -f "$VEGETA_LOADER" >/dev/null 2>&1 || true
  docker run -d --name "$VEGETA_LOADER" --network "$NETWORK" \
    -v "$bench_dir_abs:/bench-results" \
    --entrypoint sleep "$VEGETA_IMAGE" 86400 >/dev/null
  docker exec "$VEGETA_LOADER" sh -c "printf '%s' '{}' > /tmp/body.json && cat > /tmp/targets.txt <<EOF
POST $LOAD_URL
Content-Type: application/json
@/tmp/body.json
EOF"
}

# Variants share an image. Each spring-boot-* variant maps to the same docker
# image; the GC choice is injected at `docker run` time via JAVA_TOOL_OPTIONS.
image_for() {
  case "$1" in
    spring-boot*)      echo parcel-api-spring-boot ;;
    quarkus-native-g1) echo parcel-api-quarkus-native-g1 ;;
    quarkus-native)    echo parcel-api-quarkus-native ;;
    quarkus*)          echo parcel-api-quarkus-jvm ;;
    rust)              echo parcel-api-rust ;;
    c)                 echo parcel-api-c ;;
    go)                echo parcel-api-go ;;
    node)              echo parcel-api-node ;;
    *) echo "unknown variant: $1" >&2; exit 1 ;;
  esac
}

# JAVA_TOOL_OPTIONS for each GC. Rust gets no flags.
gc_opts_for() {
  case "$1" in
    # Native variants bake the GC in at build time (--gc=G1 etc.); the binary
    # ignores JAVA_TOOL_OPTIONS, so match these before the *-g1 glob below
    # (which "quarkus-native-g1" would otherwise hit).
    quarkus-native*) echo "" ;;
    *-g1)            echo "-XX:+UseG1GC" ;;
    *-parallel)      echo "-XX:+UseParallelGC" ;;
    *-zgc)           echo "-XX:+UseZGC" ;;
    *-shenandoah)    echo "-XX:+UseShenandoahGC" ;;
    *)               echo "" ;;
  esac
}

# Clean-build a variant's docker image. JVM apps clear Gradle's build/ first;
# Rust uses cargo-chef in the Dockerfile so the dep layer caches naturally,
# and we pass APP_BUILD_ID via --build-arg to invalidate only the app-code
# layer. Both setups now rebuild just the application artifacts while reusing
# the dependency cache, mirroring `gradle clean` semantics.
clean_build_image() {
  local variant=$1
  local image
  image=$(image_for "$variant")
  case "$variant" in
    spring-boot*)
      (cd parcel-api-spring-boot && ./gradlew clean bootJar -q) >&2
      docker build --no-cache --load -t "$image" \
        -f parcel-api-spring-boot/Dockerfile . >&2
      ;;
    quarkus-native-g1)
      docker build --load -t "$image" \
        --build-arg APP_BUILD_ID="$(date +%s%N)" \
        -f parcel-api-quarkus-native/Dockerfile.native-g1 . >&2
      ;;
    quarkus-native)
      docker build --load -t "$image" \
        --build-arg APP_BUILD_ID="$(date +%s%N)" \
        -f parcel-api-quarkus-native/Dockerfile.native . >&2
      ;;
    quarkus*)
      (cd parcel-api-quarkus-jvm && ./gradlew clean build -x test -q) >&2
      docker build --no-cache --load -t "$image" \
        -f parcel-api-quarkus-jvm/Dockerfile . >&2
      ;;
    rust)
      docker build --load -t "$image" \
        --build-arg APP_BUILD_ID="$(date +%s%N)" \
        -f parcel-api-rust/Dockerfile . >&2
      ;;
    c)
      docker build --load -t "$image" \
        --build-arg APP_BUILD_ID="$(date +%s%N)" \
        -f parcel-api-c/Dockerfile . >&2
      ;;
    go)
      docker build --load -t "$image" \
        --build-arg APP_BUILD_ID="$(date +%s%N)" \
        -f parcel-api-go/Dockerfile . >&2
      ;;
    node)
      docker build --load -t "$image" \
        --build-arg APP_BUILD_ID="$(date +%s%N)" \
        -f parcel-api-node/Dockerfile . >&2
      ;;
  esac
}

# Where each image's deployable lives.
artifact_path_for() {
  case "$1" in
    spring-boot*)      echo /app.jar ;;
    quarkus-native-g1) echo /usr/local/bin/parcel-api ;;
    quarkus-native)    echo /usr/local/bin/parcel-api ;;
    quarkus*)          echo /app ;;
    rust)              echo /usr/local/bin/parcel-api ;;
    c)                 echo /usr/local/bin/parcel-api ;;
    go)                echo /usr/local/bin/parcel-api ;;
    node)              echo /app ;;
  esac
}

# Total bytes of $2 inside image $1. Works for files and directories — extracts
# via `docker cp` so distroless images without a shell are fine.
artifact_bytes() {
  local image=$1 path=$2 cid tmp name size
  cid=$(docker create "$image")
  tmp=$(mktemp -d)
  docker cp "$cid:$path" "$tmp/" >/dev/null 2>&1
  docker rm "$cid" >/dev/null
  name=$(basename "$path")
  if [ -d "$tmp/$name" ]; then
    size=$(find "$tmp/$name" -type f -exec stat -f %z {} \; 2>/dev/null \
           | awk '{s += $1} END {print s+0}')
  else
    size=$(stat -f %z "$tmp/$name" 2>/dev/null || echo 0)
  fi
  rm -rf "$tmp"
  echo "${size:-0}"
}

# Total image size in bytes (uncompressed, as Docker reports).
image_bytes() { docker image inspect --format '{{.Size}}' "$1"; }

to_mib() { awk -v b="$1" 'BEGIN { printf "%.1f", b / 1048576 }'; }

# Wall-clock milliseconds (Python because macOS `date` lacks %N).
now_ms() { python3 -c 'import time; print(time.time_ns() // 1_000_000)'; }

# Background sampler: 1 Hz, writes "elapsed_ms,cores,rss_mib" rows. Polls the
# Docker stats endpoint for CPU + memory; throughput comes from wrk's own
# output instead, since wrk doesn't expose per-second progress.
sampler() {
  local out=$1 t0_ms=$2
  python3 - "$out" "$t0_ms" "$CONTAINER" >/dev/null 2>&1 <<'PY' &
import sys, time, json, socket, http.client

out_path, t0_ms, container = sys.argv[1:]
t0_ms = int(t0_ms)

class UnixHTTPConnection(http.client.HTTPConnection):
    def __init__(self, sock_path):
        super().__init__("docker")
        self.sock_path = sock_path
    def connect(self):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.sock_path)

def fetch_stats():
    conn = UnixHTTPConnection("/var/run/docker.sock")
    conn.request("GET", f"/containers/{container}/stats?stream=false&one-shot=true")
    data = json.loads(conn.getresponse().read())
    conn.close()
    return data

last_cpu = last_wall = None

with open(out_path, "w") as f:
    f.write("elapsed_ms,cores,rss_mib\n")
    while True:
        try:
            d = fetch_stats()
        except Exception:
            time.sleep(1); continue
        wall_ns = time.time_ns()
        cpu_ns = d["cpu_stats"]["cpu_usage"]["total_usage"]
        mem = d.get("memory_stats", {})
        usage = mem.get("usage", 0)
        inactive = mem.get("stats", {}).get("inactive_file", 0)
        rss_mib = max(usage - inactive, 0) / 1048576.0
        if last_cpu is None:
            cores = 0.0
        else:
            d_cpu = cpu_ns - last_cpu
            d_wall = wall_ns - last_wall
            cores = d_cpu / d_wall if d_wall > 0 else 0.0
        now_t = time.monotonic()
        elapsed_ms = (wall_ns // 1_000_000) - t0_ms
        if last_cpu is not None:
            f.write(f"{elapsed_ms},{cores:.3f},{rss_mib:.2f}\n")
            f.flush()
        last_cpu, last_wall = cpu_ns, wall_ns
        sleep_for = 1.0 - (time.monotonic() - now_t)
        if sleep_for > 0:
            time.sleep(sleep_for)
PY
  echo $!
}

# Cumulative container CPU nanoseconds + wall-clock nanoseconds on one line.
# Reads the Docker Engine API one-shot stats endpoint so the value is the raw
# cgroup counter, not a derived percentage — diffing two samples over a known
# wall-clock window gives an exact "average CPU cores used" for that window.
sample() {
  curl -sf --unix-socket /var/run/docker.sock \
    "http://docker/containers/${CONTAINER}/stats?stream=false&one-shot=true" \
    | python3 -c 'import sys, json, time; d = json.load(sys.stdin); print(d["cpu_stats"]["cpu_usage"]["total_usage"], time.time_ns())'
}

# Average CPU cores used between two samples. args: cpu0 cpu1 wall0 wall1 (ns).
cores_used() {
  awk -v c0="$1" -v c1="$2" -v w0="$3" -v w1="$4" \
    'BEGIN { d = w1 - w0; if (d <= 0) print "0.00"; else printf "%.2f\n", (c1 - c0) / d }'
}

# Container RSS in MiB. Parses "12.34MiB / 1.234GiB" from docker stats.
mem_mib() {
  docker stats --no-stream --format '{{.MemUsage}}' "$CONTAINER" \
    | awk -F'/' '{
        v = $1
        gsub(/ /, "", v)
        num = v + 0
        unit = v; sub(/^[0-9.]+/, "", unit)
        mib = num
        if (tolower(unit) == "gib")      mib = num * 1024
        else if (tolower(unit) == "kib") mib = num / 1024
        else if (tolower(unit) == "b")   mib = num / 1024 / 1024
        printf "%.1f", mib
      }'
}

# One cold-start measurement. Times "docker run" → "/check/status returns 200".
# Single python3 invocation so the interpreter startup isn't repeated mid-measurement.
# $2 is a JAVA_TOOL_OPTIONS string injected via -e (empty for Rust).
cold_start_once() {
  local image=$1 java_opts=${2:-}
  cleanup
  python3 - "$CONTAINER" "$HOST_PORT" "$image" "$READY_URL" "$CPU_LIMIT" "$NETWORK" "$java_opts" "$MEM_LIMIT" <<'PY'
import subprocess, sys, time, urllib.request, urllib.error
container, port, image, ready_url, cpu_limit, network, java_opts, mem_limit = sys.argv[1:]
cmd = ["docker", "run", "-d", "--name", container,
       "--network", network,
       "--cpus", cpu_limit,
       "-p", f"{port}:8080"]
if mem_limit:
    cmd += ["--memory", mem_limit, "--memory-swap", mem_limit]
if java_opts:
    cmd += ["-e", f"JAVA_TOOL_OPTIONS={java_opts}"]
cmd.append(image)
start = time.time()
subprocess.check_call(cmd, stdout=subprocess.DEVNULL)
while True:
    try:
        urllib.request.urlopen(ready_url, timeout=1).read()
        break
    except (urllib.error.URLError, ConnectionError, OSError):
        time.sleep(0.01)
print(f"{time.time() - start:.3f}")
PY
}

# Run vegeta for one phase. Args: connections duration_seconds bin_filename.
# bin_filename is just the filename (e.g. "rust.warm.bin"); the loader sees it
# at /bench-results/<filename> which is bind-mounted to $BENCH_DIR on the host.
# Echoes "<total_requests> <requests_per_sec>" parsed from vegeta report.
vegeta_phase() {
  local conns=$1 secs=$2 bin_name=$3
  docker exec "$VEGETA_LOADER" vegeta attack \
    -rate=0 -workers="$conns" -max-workers="$conns" \
    -duration="${secs}s" -targets=/tmp/targets.txt \
    -output="/bench-results/${bin_name}" \
    >/dev/null 2>&1
  docker exec "$VEGETA_LOADER" vegeta report "/bench-results/${bin_name}" \
    | awk -F'[,[:space:]]+' '/^Requests/ { print $5, $7; exit }'
}

# After all 3 phases, bin per-second 200-only request counts into <variant>.rps.csv.
# Args: out_csv_path offset0_ms offset1_ms offset2_ms bin0_name bin1_name bin2_name
build_rps_csv() {
  python3 - "$VEGETA_LOADER" "$@" <<'PY'
import json, subprocess, sys, collections
loader, out, off0, off1, off2, b0, b1, b2 = sys.argv[1:]
offsets = (int(off0), int(off1), int(off2))
bins = (b0, b1, b2)

def per_sec(bin_name):
    """Count only successful (HTTP 200) responses per second, in the loader."""
    counts = collections.Counter()
    p = subprocess.run(
        ["docker", "exec", loader, "vegeta", "encode", "-to=json",
         f"/bench-results/{bin_name}"],
        check=True, capture_output=True, text=True)
    t0_ns = None
    import datetime as dt
    for line in p.stdout.splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        ts = r["timestamp"]
        d = dt.datetime.fromisoformat(ts.replace("Z", "+00:00"))
        ns = int(d.timestamp() * 1e9)
        if t0_ns is None:
            t0_ns = ns
        sec = (ns - t0_ns) // 1_000_000_000
        if r.get("code") == 200:
            counts[sec] += 1
    return counts

rows = []
for offset_ms, bin_name in zip(offsets, bins):
    for sec, n in sorted(per_sec(bin_name).items()):
        rows.append((offset_ms + sec * 1000, n))
rows.sort()
with open(out, "w") as f:
    f.write("elapsed_ms,reqs_per_sec\n")
    for em, n in rows:
        f.write(f"{em},{n}\n")
PY
}

# Linear-interpolation percentile from numbers on stdin.
pct() {
  sort -n | awk -v p="$1" '
    { a[NR] = $1 }
    END {
      n = NR
      if (n == 0) { print "n/a"; exit }
      k = (n - 1) * p / 100 + 1
      f = int(k); c = (k == f) ? f : f + 1
      if (f == c) printf "%.3f\n", a[f]
      else printf "%.3f\n", a[f] + (a[c] - a[f]) * (k - f)
    }
  '
}

bench_one() {
  local variant=$1 image times t median p95 idle warm peak art_b img_b java_opts
  local build_s=${2:-"-"}
  image=$(image_for "$variant")
  java_opts=$(gc_opts_for "$variant")
  echo "==> $variant ($image${java_opts:+, JAVA_TOOL_OPTIONS=$java_opts})" >&2

  if ! docker image inspect "$image" >/dev/null 2>&1; then
    echo "    image not found, skipping" >&2
    return 0
  fi

  art_b=$(artifact_bytes "$image" "$(artifact_path_for "$variant")")
  img_b=$(image_bytes "$image")
  echo "    artifact:  $(to_mib "$art_b") MiB ($(artifact_path_for "$variant"))" >&2
  echo "    image:     $(to_mib "$img_b") MiB" >&2
  [ "$build_s" != "-" ] && echo "    build:     ${build_s}s" >&2

  times=""
  for i in $(seq 1 "$ITERS"); do
    t=$(cold_start_once "$image" "$java_opts")
    echo "    cold-start $i/$ITERS: ${t}s" >&2
    times="$times $t"
  done
  median=$(printf '%s\n' $times | pct 50)
  p95=$(printf '%s\n' $times | pct 95)

  # Last container is still running; settle before reading idle RSS.
  sleep 3
  idle=$(mem_mib)
  echo "    idle RSS:  ${idle} MiB" >&2

  local warm_reqs warm_rps ramp_reqs ramp_rps steady_reqs steady_rps
  local total_reqs total_dur t0
  local warm_cpu peak_cpu c0 w0 c1 w1
  local bench_t0 t_warm_start t_warm_end t_ramp_start t_ramp_end t_steady_start t_steady_end sampler_pid
  local ramp_conns=$(( MAX_PARALLEL / 2 ))
  # Filenames only; vegeta sees them at /bench-results/<file> inside the loader.
  local warm_bin="${variant}.warm.bin"
  local ramp_bin="${variant}.ramp.bin"
  local steady_bin="${variant}.steady.bin"

  mkdir -p "$BENCH_DIR"
  bench_t0=$(now_ms)
  SAMPLER_PID=$(sampler "$BENCH_DIR/${variant}.csv" "$bench_t0")
  sampler_pid=$SAMPLER_PID
  sleep 1  # let sampler emit its first row before phase 1 starts

  echo "    phase 1: ${WARMUP_SECONDS}s warmup vegeta workers=1..." >&2
  t0=$(date +%s)
  t_warm_start=$(now_ms)
  read -r c0 w0 <<<"$(sample)"
  read -r warm_reqs warm_rps <<<"$(vegeta_phase 1 "$WARMUP_SECONDS" "$warm_bin")"
  read -r c1 w1 <<<"$(sample)"
  warm=$(mem_mib)
  warm_cpu=$(cores_used "$c0" "$c1" "$w0" "$w1")
  t_warm_end=$(now_ms)
  echo "    warm:  ${warm_reqs} req @ ${warm_rps} req/s, RSS ${warm} MiB, CPU ${warm_cpu} cores" >&2

  echo "    phase 2: ${RAMP_SECONDS}s ramp vegeta workers=${ramp_conns}..." >&2
  t_ramp_start=$(now_ms)
  read -r ramp_reqs ramp_rps <<<"$(vegeta_phase "$ramp_conns" "$RAMP_SECONDS" "$ramp_bin")"
  t_ramp_end=$(now_ms)
  echo "    ramp:  ${ramp_reqs} req @ ${ramp_rps} req/s" >&2

  echo "    phase 3: ${STEADY_SECONDS}s steady vegeta workers=${MAX_PARALLEL}..." >&2
  t_steady_start=$(now_ms)
  read -r c0 w0 <<<"$(sample)"
  read -r steady_reqs steady_rps <<<"$(vegeta_phase "$MAX_PARALLEL" "$STEADY_SECONDS" "$steady_bin")"
  read -r c1 w1 <<<"$(sample)"
  peak=$(mem_mib)
  peak_cpu=$(cores_used "$c0" "$c1" "$w0" "$w1")
  t_steady_end=$(now_ms)
  total_dur=$(( $(date +%s) - t0 ))
  total_reqs=$(( warm_reqs + ramp_reqs + steady_reqs ))
  echo "    peak:  ${steady_reqs} req @ ${steady_rps} req/s, RSS ${peak} MiB, CPU ${peak_cpu} cores" >&2
  echo "    total: ${total_reqs} req over ${total_dur}s (steady = ${steady_rps} req/s)" >&2

  # Bin per-second request counts across all three phases into one CSV.
  build_rps_csv "$BENCH_DIR/${variant}.rps.csv" \
    $((t_warm_start - bench_t0)) \
    $((t_ramp_start - bench_t0)) \
    $((t_steady_start - bench_t0)) \
    "$warm_bin" "$ramp_bin" "$steady_bin"
  rm -f "$BENCH_DIR/$warm_bin" "$BENCH_DIR/$ramp_bin" "$BENCH_DIR/$steady_bin"

  kill "$sampler_pid" 2>/dev/null || true
  wait "$sampler_pid" 2>/dev/null || true
  SAMPLER_PID=
  {
    echo "warm_end_ms=$((t_warm_end - bench_t0))"
    echo "ramp_end_ms=$((t_ramp_end - bench_t0))"
    echo "steady_end_ms=$((t_steady_end - bench_t0))"
    echo "warm_rps=$warm_rps"
    echo "ramp_rps=$ramp_rps"
    echo "steady_rps=$steady_rps"
    echo "cpu_limit=$CPU_LIMIT"
    echo "max_parallel=$MAX_PARALLEL"
  } > "$BENCH_DIR/${variant}.phases.txt"

  cleanup
  printf '%s %s %s %s %s %s %s %s %s %s %s %s %s %s\n' \
    "$variant" "$median" "$p95" "$idle" "$warm" "$warm_cpu" "$peak" "$peak_cpu" "$total_reqs" "$total_dur" "$steady_rps" "$art_b" "$img_b" "$build_s"
}

print_table() {
  local results
  results=$(cat)

  printf '\n'
  printf '%-22s | %-9s | %-10s | %-8s | %-10s | %-10s | %-10s | %-10s | %-10s | %-10s | %-8s | %-7s\n' \
    Variant 'Build (s)' 'Median (s)' 'p95 (s)' 'Idle (MiB)' 'Warm (MiB)' 'Warm CPU' 'Peak (MiB)' 'Peak CPU' 'Requests' 'Load (s)' 'Req/s'
  printf -- '-----------------------+-----------+------------+----------+------------+------------+------------+------------+------------+------------+----------+--------\n'
  printf '%s\n' "$results" | while IFS=' ' read -r v median p95 idle warm warm_cpu peak peak_cpu reqs dur rps art_b img_b build_s; do
    [ -z "$v" ] && continue
    printf '%-22s | %-9s | %-10s | %-8s | %-10s | %-10s | %-10s | %-10s | %-10s | %-10s | %-8s | %-7s\n' \
      "$v" "$build_s" "$median" "$p95" "$idle" "$warm" "$warm_cpu" "$peak" "$peak_cpu" "$reqs" "$dur" "$rps"
  done

  printf '\n'
  printf '%-22s | %-15s | %-15s\n' Variant 'Artifact (MiB)' 'Image (MiB)'
  printf -- '-----------------------+-----------------+----------------\n'
  printf '%s\n' "$results" | while IFS=' ' read -r v median p95 idle warm warm_cpu peak peak_cpu reqs dur rps art_b img_b build_s; do
    [ -z "$v" ] && continue
    printf '%-22s | %-15s | %-15s\n' "$v" "$(to_mib "$art_b")" "$(to_mib "$img_b")"
  done
}

if [ "$VARIANT" = "all" ]; then
  variants=("${ALL_VARIANTS[@]}")
else
  variants=("$VARIANT")
fi

mkdir -p "$BENCH_DIR"
BENCH_DIR_ABS=$(cd "$BENCH_DIR" && pwd)
echo "==> bench dir: $BENCH_DIR" >&2

echo "==> setup loader on network $NETWORK" >&2
setup_loader "$BENCH_DIR_ABS"

# Build phase: build each unique image at most once, time the work, remember
# the wall-clock cost so each variant sharing that image reports the same time.
# Plain variables (not an associative array) to keep this runnable under macOS
# bash 3.2 when invoked as `sh bench.sh`.
sb_build_s=""
qk_build_s=""
qk_native_build_s=""
qk_native_g1_build_s=""
rs_build_s=""
c_build_s=""
go_build_s=""
node_build_s=""

build_seconds_for_image() {
  case "$1" in
    parcel-api-spring-boot)    echo "${sb_build_s:--}" ;;
    parcel-api-quarkus-jvm)       echo "${qk_build_s:--}" ;;
    parcel-api-quarkus-native)    echo "${qk_native_build_s:--}" ;;
    parcel-api-quarkus-native-g1) echo "${qk_native_g1_build_s:--}" ;;
    parcel-api-rust)           echo "${rs_build_s:--}" ;;
    parcel-api-c)              echo "${c_build_s:--}" ;;
    parcel-api-go)             echo "${go_build_s:--}" ;;
    parcel-api-node)           echo "${node_build_s:--}" ;;
    *) echo "-" ;;
  esac
}

if [ "$BUILD" = "1" ]; then
  for variant in "${variants[@]}"; do
    img=$(image_for "$variant")
    case "$img" in
      parcel-api-spring-boot)    [ -n "$sb_build_s" ] && continue ;;
      parcel-api-quarkus-jvm)       [ -n "$qk_build_s" ] && continue ;;
      parcel-api-quarkus-native)    [ -n "$qk_native_build_s" ] && continue ;;
      parcel-api-quarkus-native-g1) [ -n "$qk_native_g1_build_s" ] && continue ;;
      parcel-api-rust)           [ -n "$rs_build_s" ] && continue ;;
      parcel-api-c)              [ -n "$c_build_s" ] && continue ;;
      parcel-api-go)             [ -n "$go_build_s" ] && continue ;;
      parcel-api-node)           [ -n "$node_build_s" ] && continue ;;
    esac
    echo "==> build $img" >&2
    t0=$(date +%s)
    clean_build_image "$variant"
    elapsed=$(( $(date +%s) - t0 ))
    case "$img" in
      parcel-api-spring-boot)    sb_build_s=$elapsed ;;
      parcel-api-quarkus-jvm)       qk_build_s=$elapsed ;;
      parcel-api-quarkus-native)    qk_native_build_s=$elapsed ;;
      parcel-api-quarkus-native-g1) qk_native_g1_build_s=$elapsed ;;
      parcel-api-rust)           rs_build_s=$elapsed ;;
      parcel-api-c)              c_build_s=$elapsed ;;
      parcel-api-go)             go_build_s=$elapsed ;;
      parcel-api-node)           node_build_s=$elapsed ;;
    esac
    echo "    build: ${elapsed}s" >&2
  done
fi

# Bench phase.
results=""
for variant in "${variants[@]}"; do
  img=$(image_for "$variant")
  build_s=$(build_seconds_for_image "$img")
  line=$(bench_one "$variant" "$build_s")
  results="${results:+$results
}${line}"
done
printf '%s\n' "$results" | print_table

# Generate the time-series graph if any CSVs were produced.
if compgen -G "$BENCH_DIR/*.csv" >/dev/null && command -v uv >/dev/null; then
  uv run --with matplotlib --python 3.13 graph.py "$BENCH_DIR" >&2 || true
fi
