# Rust adoption — stats & sources (2025 / 2026)

Figures collected for the "Rust adoption in 2026" slide. Most come from the 2025
survey cycle (reported late 2025 / early 2026) — the freshest data available.
Reliability is flagged: **[primary]** = original survey/source, **[secondary]** =
blog/aggregator reporting on it (verify before quoting on stage).

## Surveys & developer sentiment
- **Most admired language**, again — ~72% of developers want to keep using Rust (Stack Overflow 2025); it has topped this list every year since 2016. **[primary]** — [Stack Overflow Developer Survey 2025](https://survey.stackoverflow.co/2025/technology/)
- **45.5% of organizations** make *non-trivial use* of Rust, up from **38.7%** the year before. **[secondary/primary]** — [The New Stack](https://thenewstack.io/survey-memory-safe-rust-gains-45-of-enterprise-development/), [byteiota on the 2025 survey](https://byteiota.com/rust-2025-survey-45-5-adoption-41-6-worry-complexity/)
- **82%** of teams using Rust say it helped their company meet its goals; **daily usage ~53%** (up 4pp from 2023). **[secondary]** — [The New Stack: nearly half of companies now use Rust](https://thenewstack.io/rust-enterprise-developers/)
- **~30%** of Rust developers started within the last year; **65%** use it for side/hobby projects, **26%** in professional projects. **[primary]** — [JetBrains: State of Rust 2025](https://blog.jetbrains.com/rust/2026/02/11/state-of-rust-2025/)
- **41.6%** still worry about Rust's complexity — the honest counterweight. **[secondary]** — [byteiota](https://byteiota.com/rust-2025-survey-45-5-adoption-41-6-worry-complexity/)
- Official survey write-up (7,156 responses; hiring demand up). **[primary]** — [2025 State of Rust Survey results (Rust Blog)](https://blog.rust-lang.org/2026/03/02/2025-State-Of-Rust-Survey-results/)

## Developer population
- **2.2M+ developers** used Rust in the past 12 months; **~709k** call it their primary language; **68.75%** increase in commercial use 2021–2024. **[secondary]** (SlashData-derived figures) — [ZenRows: Is Rust still surging?](https://www.zenrows.com/blog/rust-popularity)

## Rankings & momentum
- **TIOBE**: climbed to **#8** (Nov 2025), up from #17 a year earlier; hit an all-time high of #13 in Feb 2025. **[secondary]** — [Tech Bytes / TIOBE Nov 2025](https://techbytes.app/tech-pulse-daily/2025/november/14/), [language-stats roundup](https://rockstardeveloperuniversity.com/programming-language-statistics/)
  - *Verified 2026-06-11:* the techbytes URL is a rolling "daily pulse" page — it now shows newer content (still consistent: Rust #8, from #17, 1.98% rating). For a stable on-stage reference use [tiobe.com/tiobe-index](https://www.tiobe.com/tiobe-index/) as the primary source. Used on the "Climbing the TIOBE index" slide.
- **RedMonk (Jan 2025)**: Rust at **#19**. **[primary]** — [RedMonk rankings](https://redmonk.com/sogrady/2025/06/18/language-rankings-1-25/)
- **GitHub Octoverse**: Rust growing ~**40% YoY**. **[primary]** — [Octoverse](https://octoverse.github.com/)

## Production & industry milestones
- **Linux kernel made Rust permanent** in 2025 (Kernel Maintainer Summit, Tokyo) — the experimental phase is over. **[primary]** — [Phoronix](https://www.phoronix.com/news/Rust-To-Stay-Linux-Kernel), [devclass](https://devclass.com/2025/12/15/rust-boosted-by-permanent-adoption-for-linux-kernel-code/)
- **Android 16** ships Rust *inside the kernel* on millions of devices (e.g. the ashmem shared-memory allocator rewritten in Rust). **[secondary]** — [DesdeLinux](https://blog.desdelinux.net/en/linux-kernel-rust-official-android-16-drivers-drm-debate/)
- **Ubuntu 25.10** ships Rust **coreutils** as default; Fedora/Ubuntu enabling Rust in default kernels. **[secondary]** — [DesdeLinux](https://blog.desdelinux.net/en/linux-kernel-rust-official-android-16-drivers-drm-debate/)
- **AWS Firecracker** (Lambda) runs **trillions of executions/month** on Rust; also Lambda, S3. **[primary]** — [AWS: Sustainability with Rust](https://aws.amazon.com/blogs/opensource/sustainability-with-rust/)

## Jobs & hiring
- Rust job postings **up ~35% YoY**; avg salary ~**$130k**, senior roles to **$235k** — a **15–20% premium** over Go/Python/Java, driven by a talent shortage (2.2M users, ~709k primary). **[secondary]** — [byteiota: Rust salaries hit $130k](https://byteiota.com/rust-dev-salaries-hit-130k-job-market-explodes-35/)
