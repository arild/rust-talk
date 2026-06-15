#!/usr/bin/env python3
"""
Plot the time-series CSVs produced by bench.sh.

Each variant writes a CSV:
  elapsed_ms,cores,rss_mib
and a phases file with the warm/ramp/steady boundary timestamps. We overlay all
three variants on the same axes (each run starts at t=0 relative to its own
bench), with vertical lines at the phase boundaries (taken from the first
variant; they're approximately equal for all three). Throughput numbers come
from wrk's output and are reported per phase in the bench table — not in the
time-series here.
"""

import csv
import sys
from pathlib import Path

try:
    import matplotlib.pyplot as plt
except ImportError:
    sys.stderr.write(
        "matplotlib not installed. Install with: pip3 install --user matplotlib\n"
    )
    sys.exit(1)

FAMILY_COLORS = {
    "spring-boot": "#2ca02c",   # green
    "quarkus":     "#1f77b4",   # blue
    "rust":        "#d62728",   # red
}
# Distinguish GC variants of the same family by line style.
GC_LINESTYLES = {
    "g1":         "-",
    "parallel":   "--",
    "zgc":        ":",
    "shenandoah": "-.",
}


def variant_color(variant: str) -> str:
    for family, color in FAMILY_COLORS.items():
        if variant.startswith(family):
            return color
    return "#666666"


def variant_linestyle(variant: str) -> str:
    for gc, style in GC_LINESTYLES.items():
        if variant.endswith("-" + gc):
            return style
    return "-"


def load_csv(path: Path):
    times, cores, rss = [], [], []
    with open(path) as f:
        reader = csv.DictReader(f)
        for row in reader:
            times.append(int(row["elapsed_ms"]) / 1000.0)
            cores.append(float(row["cores"]))
            rss.append(float(row["rss_mib"]))
    return times, cores, rss


def load_phases(path: Path):
    out = {}
    for line in path.read_text().splitlines():
        k, _, v = line.partition("=")
        try:
            out[k] = int(v)
        except ValueError:
            try:
                out[k] = float(v)
            except ValueError:
                out[k] = v
    return out


def load_rps_csv(path: Path):
    """Per-second rps time series produced by vegeta -> build_rps_csv."""
    times, rps = [], []
    with open(path) as f:
        reader = csv.DictReader(f)
        for row in reader:
            times.append(int(row["elapsed_ms"]) / 1000.0)
            rps.append(float(row["reqs_per_sec"]))
    return times, rps


def rps_steps(ph):
    """Piecewise-constant per-phase fallback if no per-second rps CSV exists."""
    if not all(k in ph for k in ("warm_rps", "ramp_rps", "steady_rps")):
        return None, None
    warm_end = ph["warm_end_ms"] / 1000
    ramp_end = ph["ramp_end_ms"] / 1000
    steady_end = ph["steady_end_ms"] / 1000
    xs = [0, warm_end, warm_end, ramp_end, ramp_end, steady_end]
    ys = [ph["warm_rps"], ph["warm_rps"],
          ph["ramp_rps"], ph["ramp_rps"],
          ph["steady_rps"], ph["steady_rps"]]
    return xs, ys


def main():
    bench_dir = Path(sys.argv[1] if len(sys.argv) > 1 else "./bench-results")
    if not bench_dir.is_dir():
        sys.stderr.write(f"no such dir: {bench_dir}\n")
        sys.exit(1)

    fig, (ax_cpu, ax_mem, ax_rps) = plt.subplots(3, 1, figsize=(13, 10), sharex=True)

    phases = None
    cpu_limit = None
    max_parallel = None

    for csv_path in sorted(bench_dir.glob("*.csv")):
        if csv_path.name.endswith(".rps.csv"):
            continue  # rps CSVs are handled separately below
        variant = csv_path.stem
        ph_path = bench_dir / f"{variant}.phases.txt"
        rps_csv = bench_dir / f"{variant}.rps.csv"
        t, cores, rss = load_csv(csv_path)
        color = variant_color(variant)
        ls = variant_linestyle(variant)
        ax_cpu.plot(t, cores, label=variant, color=color, linestyle=ls, linewidth=1.5)
        ax_mem.plot(t, rss,   label=variant, color=color, linestyle=ls, linewidth=1.5)
        if rps_csv.exists():
            rt, rps = load_rps_csv(rps_csv)
            ax_rps.plot(rt, rps, label=variant, color=color, linestyle=ls, linewidth=1.5)
        if ph_path.exists():
            ph = load_phases(ph_path)
            if phases is None:
                phases = ph
                cpu_limit = ph.get("cpu_limit")
                max_parallel = ph.get("max_parallel")
            if not rps_csv.exists():
                xs, ys = rps_steps(ph)
                if xs is not None:
                    ax_rps.plot(xs, ys, label=variant, color=color, linestyle=ls, linewidth=1.5)

    # Phase boundaries + shading.
    if phases:
        warm_end = phases["warm_end_ms"] / 1000
        ramp_end = phases["ramp_end_ms"] / 1000
        steady_end = phases["steady_end_ms"] / 1000
        for ax in (ax_cpu, ax_mem, ax_rps):
            ax.axvspan(0, warm_end,            color="#f5f5f5")
            ax.axvspan(warm_end, ramp_end,     color="#fffbe6")
            ax.axvspan(ramp_end, steady_end,   color="#fff0f0")
            for x in (warm_end, ramp_end, steady_end):
                ax.axvline(x, color="#888", linestyle="--", linewidth=0.7)
        # Phase labels along the top of the CPU plot.
        ymax = ax_cpu.get_ylim()[1]
        labels = [
            ("warmup c=1", warm_end / 2),
            ("ramp c={}".format(max_parallel // 2 if max_parallel else "?"),
             (warm_end + ramp_end) / 2),
            ("steady c={}".format(max_parallel), (ramp_end + steady_end) / 2),
        ]
        for txt, xpos in labels:
            ax_cpu.text(xpos, ymax * 0.94, txt, ha="center", fontsize=10, color="#555")

    if cpu_limit is not None:
        ax_cpu.axhline(float(cpu_limit), color="#888", linestyle=":", linewidth=1.0)
        ax_cpu.text(0.5, float(cpu_limit) + 0.05,
                    f"limit = {cpu_limit} cores",
                    color="#666", fontsize=9)

    ax_cpu.set_ylabel("CPU (cores)")
    ax_mem.set_ylabel("RSS (MiB)")
    ax_rps.set_ylabel("Requests / s")
    ax_rps.set_xlabel("Time (s, since container start)")

    ax_cpu.set_title(
        f"parcel-api benchmark — CPU, RSS, throughput "
        f"(vegeta workers=1 → {max_parallel // 2 if max_parallel else '?'} → "
        f"{max_parallel}, --cpus={cpu_limit})"
    )

    for ax in (ax_cpu, ax_mem, ax_rps):
        ax.grid(True, alpha=0.3)
        ax.legend(loc="upper left", ncol=2, fontsize=8)

    ax_mem.set_yscale("log")
    ax_mem.set_ylabel("RSS (MiB, log)")

    plt.tight_layout()
    out = bench_dir / "bench-graph.png"
    plt.savefig(out, dpi=130)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
