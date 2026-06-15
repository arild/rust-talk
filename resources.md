# Resources

## General Rust performance benchmarks
- [TechEmpower Web Framework Benchmarks](https://www.techempower.com/benchmarks/) — independent, third-party benchmarks. Rust frameworks (actix, axum) routinely sit at the top; Spring is far down the list.
- [How can Rust be so fast in the TechEmpower benchmarks?](https://kerkour.com/rust-fast-techempower-web-framework-benchmarks) — explains *why* Rust dominates.
- [Programming Language Benchmarks: Rust vs Java](https://programming-language-benchmarks.vercel.app/rust-vs-java) — head-to-head micro-benchmarks across many tasks, with source per benchmark.

## Rust vs JVM specifically
- [Rust vs Java in 2025: Benchmarking Memory and Latency to Expose the JVM's Hidden Cost](https://medium.com/@premchandak_11/rust-vs-java-in-2025-benchmarking-memory-and-latency-to-expose-the-jvms-hidden-cost-471d9ddef0f8)
- [Rust vs Spring Boot vs Quarkus: The Performance Truth Nobody Talks About](https://medium.com/javarevisited/rust-vs-spring-boot-vs-quarkus-the-performance-truth-nobody-talks-about-09941b196f8e) — includes Quarkus/GraalVM.
- [JetBrains: Rust vs Java](https://blog.jetbrains.com/rust/2025/08/01/rust-vs-java/) — balanced, vendor-neutral framing.

## Real-world case studies
- [Why Discord is switching from Go to Rust](https://discord.com/blog/why-discord-is-switching-from-go-to-rust) — the canonical "GC pauses killed us" story.
- [I Rewrote a Java Microservice in Rust: CPU & RAM Stats](https://medium.com/@kushalburad07/i-rewrote-a-java-microservice-in-rust-here-are-the-cpu-ram-stats-8849ef45026d)
- [Don't choose Spring Boot & OpenJDK for your Kubernetes microservice](https://medium.com/@gael.lm/dont-choose-spring-boot-openjdk-for-your-kubernetes-microservice-9a2168b95e31)

## Rust vs Go
- [Rust vs Go — Bitfield Consulting](https://bitfieldconsulting.com/posts/rust-vs-go) — calm, balanced comparison from a Go author.
- [JetBrains: Rust vs Go (2025)](https://blog.jetbrains.com/rust/2025/06/12/rust-vs-go/) — concise side-by-side.
- [Programming Language Benchmarks: Go vs Rust](https://programming-language-benchmarks.vercel.app/go-vs-rust) — micro-benchmarks with source.

## Big-tech adoption
- [Cloudflare: How we built Pingora](https://blog.cloudflare.com/how-we-built-pingora-the-proxy-that-connects-cloudflare-to-the-internet/) — replaced NGINX, less than half the CPU and memory.
- [Cloudflare: Cloudflare just got faster and more secure, powered by Rust (FL2)](https://blog.cloudflare.com/20-percent-internet-upgrade/)
- [Cloudflare: Introducing Foundations, our open source Rust service foundation library](https://blog.cloudflare.com/introducing-foundations-our-open-source-rust-service-foundation-library/)
- [AWS: Sustainability with Rust](https://aws.amazon.com/blogs/opensource/sustainability-with-rust/) — Firecracker, Lambda, S3.
- [Companies that use Rust in production](https://litslink.com/blog/companies-that-use-rust-language) — name-drop list.

## Language fundamentals
- [The Rust Book — Introduction](https://doc.rust-lang.org/book/ch00-00-introduction.html) — canonical.
- [The Rust Book — Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html) — best single chapter to point listeners to.
- [Rust's Language Ergonomics Initiative (Rust Blog, 2017)](https://blog.rust-lang.org/2017/03/02/lang-ergonomics/) — the *why* behind ergonomics decisions.

## Containers & Kubernetes
- [Creating Lightweight Docker Images with Rust](https://medium.com/@pabloperezaradros/creating-lightweight-docker-images-with-rust-0db47cb014a9) — multi-stage builds, ~12 MB final image.
- [How to Package Rust Applications Into Minimal Docker Containers](https://alexbrand.dev/post/how-to-package-rust-applications-into-minimal-docker-containers/) — practical, scratch/distroless.
- [Rust — Fast + Small Docker Image Builds](https://shaneutt.com/blog/rust-fast-small-docker-image-builds/)

## Rust + AI / coding agents

The core argument: when an agent generates code, Rust's compiler gives it a tight,
machine-readable feedback loop — `cargo check` fails with a *structured, specific*
error (often with a suggested fix), the agent parses it and corrects on the next pass.
In Python/JS the same mistake (type confusion, a stray `None`) compiles fine and only
blows up at runtime, where the agent gets no early signal. So Rust's strictness, usually
called a learning-curve cost, becomes an asset for AI-assisted development.

### The case for (pro)
- [@CtrlAltDwayne on X — Rust is well-suited to AI coding agents](https://x.com/CtrlAltDwayne/status/2032388050584736157?s=20) — argues Rust's strong type system and compiler errors give coding agents a tight feedback loop, making it a particularly good fit for AI-assisted development right now.
- [How Rust's Compiler Catches What Coding Agents Get Wrong — Marc Love](https://marclove.com/blog/2025-12-13-rust-feedback-loop-catches-claude-code-hallucinations-dead-code-bugs/) — concrete: `cargo check` catching Claude Code hallucinations and dead-code bugs.
- [Rust and LLMs: The Compiler Does What Code Review Shouldn't Have To — DEV (arezvov)](https://dev.to/arezvov/rust-and-llms-the-compiler-does-what-code-review-shouldnt-have-to-3ia4) — Python/JS errors pass silently; Rust rejects until every type resolves and every error case is handled.
- [Choosing Rust for LLM-Generated Code — RunMat](https://runmat.com/blog/rust-llm-training-distribution) — the "tighter training distribution + compiler safety net" argument; also honest about the data-sparsity downside.
- [Why Learning Rust Still Matters in the Age of LLM Coding Agents — reltech](https://reltech.substack.com/p/why-learning-rust-still-matters-in) — why the language still pays off even when agents write most of the code.

### Research & benchmarks (the evidence)
- [RustAssistant: Using LLMs to Fix Compilation Errors in Rust Code — Microsoft Research, ICSE 2025 (PDF)](https://www.microsoft.com/en-us/research/wp-content/uploads/2024/08/paper.pdf) — ~74% accuracy fixing real-world GitHub compile errors, 91–93% on focused benchmarks.
- [DevQualityEval v1.1: the best LLM for Rust coding — Symflower](https://symflower.com/en/company/blog/2025/dev-quality-eval-v1.1-openai-gpt-4.1-nano-is-the-best-llm-for-rust-coding/) — head-to-head model benchmark on generating usable Rust.
- [RustEvo²: An Evolving Benchmark for LLM-based Rust Code Generation — arXiv](https://arxiv.org/abs/2503.16922) — academic benchmark tracking API evolution.

### The counterpoint (against / nuance)
- Rust has **less training data** than Python/JS/Java, so base generation is weaker — in one 2025 benchmark ~58% of models still struggled to produce immediately usable Rust. The pro-Rust rebuttal is that the corpus is *cleaner* (cargo layout, rustfmt, Clippy, a test/CI culture) and the compiler loop compensates for quantity with quality. See the RunMat and Symflower links above, which cover both sides.

## Memory safety & security — the core case for Rust (pro)
- [What is memory safety and why does it matter? — Prossimo (ISRG)](https://www.memorysafety.org/docs/memory-safety/) — credible, vendor-neutral primer from the group behind Let's Encrypt.
- [Why so many are switching to a memory-safe language (Rust) — Fluid Attacks](https://fluidattacks.com/blog/switch-memory-safe-language-rust) — the ~70% of CVEs are memory-safety bugs argument (Microsoft/Chrome/Android data), plus NSA/White House advisories.
- [Rust Code Delivers Better Security, Also Streamlines DevOps — Dark Reading](https://www.darkreading.com/application-security/rust-code-delivers-better-security-streamlines-devops) — Google Android data: far fewer bugs, faster review, more stable fixes.

## More 2025 rewrite case studies (pro)
- [Cloudflare Rust Rewrite Enables CDN Performance + Security gains — InfoQ (Oct 2025)](https://www.infoq.com/news/2025/10/cloudflare-rust-proxy/) — FL2: ~25% faster, half the CPU, less than half the memory vs the old C/Lua proxy.
- [2x Performance, $300k Savings: Rewriting a Critical Service in Rust](https://wxiaoyun.com/blog/rust-rewrite-case-study/) — a *selective* rewrite of only the hot CPU-bound endpoints; good "don't big-bang it" lesson.

## Critical takes & arguments against Rust (against)
- [My negative views on Rust — Chris Done](https://chrisdone.com/posts/rust/) — a well-known, articulate critique of the language's complexity and ergonomics.
- [Stop Making Me Memorize The Borrow Checker — Erik McClure](https://erikmcclure.com/blog/stop-making-me-memorize-borrow-checker/) — lifetimes are "contagious" like async; small changes force re-architecting.
- [The borrowchecker is what I like the least about Rust — Jakob Nissen (Viral Instruction)](https://viralinstruction.com/posts/borrowchecker/) — pulling one loose thread forces unspooling half your code.
- [Async Rust Is A Bad Language — Bit Bashing](https://bitbashing.io/async-rust.html) — a pointed critique of the async model and its sharp edges.
- [The State of Async Rust: Runtimes — corrode](https://corrode.dev/blog/async/) — the runtime-coupling/ecosystem-fragmentation problem (Tokio lock-in), from a Rust consultancy.
- [Rust Is Hard, Or: The Misery of Mainstream Programming — Hirrolot](https://hirrolot.github.io/posts/rust-is-hard-or-the-misery-of-mainstream-programming.html) — on fighting type-system inexpressiveness with Box/Pin/Arc.
- [The dark side of Rust Language — ilegra](https://medium.com/@ilegra/the-dark-side-of-rust-language-4fe2b9c2faf3) — slow compile times, the six string types, steep learning curve, small hiring pool.

## Balanced — when Rust is (and isn't) the right tool
- [Why Not Rust? — matklad (rust-analyzer author)](https://matklad.github.io/2020/09/20/why-not-rust.html) — the most respected even-handed take; a Rust insider on the real costs.
- [When not to use Rust? — Sylvain Kerkour](https://kerkour.com/why-not-rust) — from a prolific Rust author; honest about where Rust loses.
- [Using Rust at a startup: A cautionary tale — Matt Welsh](https://mdwdotla.medium.com/using-rust-at-a-startup-a-cautionary-tale-42ab823d9454) — velocity vs. safety; why Rust can be the wrong call when time-to-market dominates.
- [Why Rust's learning curve seems harsh, and ideas to reduce it — Nicole Tietz](https://ntietz.com/blog/rust-resources-learning-curve/) — diagnoses *why* it's hard, fairly.
- [Flattening Rust's Learning Curve — corrode](https://corrode.dev/blog/flattening-rusts-learning-curve/) — counterpoint with concrete mitigations.
- [Rust 2025 Survey: 45.5% adoption, 41.6% worry about complexity — byteiota](https://byteiota.com/rust-2025-survey-45-5-adoption-41-6-worry-complexity/) — survey data on the complexity concern straight from the community.

---

## Does Rust's Memory Model Give You Extra Security Over the JVM?

A comparison written for the talk *"Introduction to Rust: Efficiency Beyond the
JVM."* The question: **how much extra security, if any, does Rust's memory model
give you compared to Java or Kotlin on the JVM?**

### TL;DR

The famous "Rust eliminates ~70% of security vulnerabilities" story is about
replacing **C and C++** — not Java or Kotlin. The JVM is **already a
memory-safe platform**, so most of Rust's headline safety advantage disappears
when the baseline is the JVM rather than C/C++.

Against the JVM, Rust still adds a few genuine but *modest* security benefits:

1. **Compile-time data-race freedom** (the biggest one).
2. A **smaller runtime attack surface** (no large JVM/JIT trusted base).
3. **No null** (Kotlin already matches this; Java does not).

But the vulnerabilities that actually get exploited in business services —
injection, broken auth, insecure deserialization, supply-chain, misconfig — are
**language-agnostic**. Rust does not protect you from those any more than the
JVM does. So: **for a typical Posten backend service, security is not the reason
to choose Rust over Kotlin. Efficiency and startup time are.**

---

### 1. Both Rust and the JVM are memory-safe

This is the crucial point that's easy to miss. The C/C++ → Rust narrative does
not transfer to Java/Kotlin → Rust, because the JVM already prevents the same
classes of memory errors:

| Memory error class            | C/C++       | JVM (Java/Kotlin)        | Rust (safe)              |
|-------------------------------|-------------|--------------------------|--------------------------|
| Buffer overflow / OOB access  | ❌ allowed  | ✅ prevented (bounds checks) | ✅ prevented           |
| Use-after-free                | ❌ allowed  | ✅ prevented (GC)        | ✅ prevented (ownership) |
| Double-free                   | ❌ allowed  | ✅ prevented (GC)        | ✅ prevented             |
| Dangling pointers             | ❌ allowed  | ✅ prevented (GC)        | ✅ prevented (borrowck)  |
| Null dereference              | ❌ allowed  | ⚠️ NPE (Java) / safe (Kotlin) | ✅ no null type     |
| **Data races**                | ❌ allowed  | ❌ **allowed**           | ✅ **prevented at compile time** |

The JVM achieves safety at **runtime** (bytecode verifier, array bounds checks,
garbage collector). Rust achieves it at **compile time** (ownership + borrow
checker), with no GC and no runtime cost. Different mechanism, but the *memory
safety outcome is largely the same* — except for one row.

The academic paper *"Memory Errors and Memory Safety: A Look at Java and Rust"*
(IEEE Security & Privacy, 2023) makes exactly this point: Java already
eliminates buffer overflows, use-after-free, and double-free via bounds checking
and GC. Rust matches that **and additionally prevents data races**, which Java
permits.

### 2. Where the "70% of vulnerabilities" stat comes from

These widely-cited numbers all describe **memory-unsafe (C/C++) codebases**:

- **Microsoft:** ~70% of CVEs assigned each year are memory-safety issues.
- **Google / Chromium:** ~70% of serious security bugs are memory-safety
  problems; >65% of high/critical bugs in Chrome and Android.
- **Android + Rust:** memory-safety share of vulnerabilities fell from **76% in
  2019 to under 20% by 2025** as new code shifted to Rust — and Google reports a
  **~1000× lower memory-safety bug density** in its Rust code vs its C/C++ code.

That last figure is dramatic and real — but it is *Rust vs C/C++*. A
Spring Boot service written in Kotlin was never in the population of code
producing those 70% memory-safety CVEs in the first place. **You cannot claim a
70% vulnerability reduction by rewriting Kotlin in Rust**, because the Kotlin
code didn't have those memory bugs to begin with.

### 3. What Rust *genuinely* adds over the JVM

#### 3.1 Data-race freedom (the strongest argument)

This is the one memory-safety property the JVM lacks. Java and Kotlin are
memory-safe but **not data-race-free**: two threads can race on shared mutable
state. On the JVM this won't corrupt the heap (the GC and memory model keep it
"safe"), but it can produce torn reads, stale values, and logic bugs — which can
become security bugs (e.g. a check-then-act auth flaw, or a shared buffer
leaking one user's data to another).

Rust prevents this **at compile time** via the `Send`/`Sync` marker traits and
the ownership/borrow rules: shared mutable aliasing across threads simply does
not compile ("fearless concurrency"). Java relies on developer discipline
(`synchronized`, `volatile`, `java.util.concurrent`) that the compiler cannot
enforce.

> Verdict: a real, language-level security advantage — but it bites hardest in
> heavily concurrent, shared-state code, which is exactly the kind of code most
> Spring services *avoid* (stateless request handlers, thread-per-request).

#### 3.2 Smaller runtime / trusted computing base

A Rust service is an AOT-compiled native binary. A JVM service ships and runs on
top of a large, complex runtime: the JVM itself, the JIT compiler, and the class
library — all part of your trusted computing base, all occasionally the subject
of their own CVEs. Less runtime machinery = less to attack. (This overlaps with
the talk's efficiency theme: no JVM also means faster startup and lower memory.)

Caveat: in practice, the JVM's *own* memory bugs are rarely how real systems get
breached — see §4.

#### 3.3 No null

Rust has no null; absence is modelled with `Option<T>` and must be handled.
**Kotlin already gives you this** via nullable types, so Rust only wins here
against *Java*, not Kotlin. NullPointerException is a reliability issue more than
a security one anyway.

### 4. Where Rust does NOT help (the honest caveats)

#### 4.1 Most exploited vulnerabilities aren't memory safety

The bugs that actually get business services breached are overwhelmingly
**language-agnostic**: SQL/command injection, broken authentication/authZ,
insecure deserialization, SSRF, secrets exposure, and security
misconfiguration (the OWASP Top 10). Analyses of *actually-exploited* CVEs (e.g.
Horizon3's review of 2023's Known Exploited Vulnerabilities) conclude that
memory safety is **not** the dominant category among real-world exploited
vulns — logic and design flaws are. **Rust gives you zero protection against
these.** A SQL injection in Rust is just as exploitable as one in Kotlin.

The JVM's own worst incidents fit this pattern: **Log4Shell (CVE-2021-44228)**
and the family of Java deserialization RCEs were **not** memory-corruption bugs.
They were design/feature-abuse flaws — and Rust's memory model would not have
prevented an equivalent design mistake in a Rust logging library.

#### 4.2 `unsafe` is an escape hatch

Rust's guarantees hold for *safe* Rust. The `unsafe` keyword (needed for FFI,
hardware access, some high-performance data structures) opts out of the borrow
checker — and a bug in unsafe code can reintroduce memory corruption that "can't
happen" in Rust. Vulnerabilities in `unsafe`-heavy crates are accordingly
treated as higher severity.

#### 4.3 Supply chain risk exists in both ecosystems

Both worlds pull in large dependency trees: crates.io for Rust, Maven Central
for the JVM. Rust tends toward *many small* crates, widening the supply-chain
surface. Rust's answer is the **RustSec advisory database** + `cargo audit`;
the JVM's is OWASP Dependency-Check / Snyk / `gradle dependencyCheck`. Neither
language is inherently safer here — Log4Shell *was* a supply-chain/dependency
incident. Cargo itself has had its own advisories (e.g. CVE-2026-33056).

### 5. Conclusion for the talk

**Is there extra security in choosing Rust over Java/Kotlin? A little — but not
the headline-grabbing kind, and not the reason to switch.**

- The **memory-safety** argument that makes Rust compelling over **C/C++ largely
  evaporates against the JVM**, because Java and Kotlin are already memory-safe.
- Rust's real, JVM-relative security wins are **compile-time data-race freedom**,
  a **smaller runtime attack surface**, and **no null** (the last of which Kotlin
  already has).
- The vulnerabilities that actually compromise services — injection, auth,
  deserialization, supply chain, misconfig — are **language-agnostic**, and Rust
  offers no advantage there.
- `unsafe`, FFI, and supply-chain risk mean Rust is **not unconditionally** more
  secure even on memory safety.

**Framing suggestion:** Position Rust's win over the JVM as **efficiency and
startup time** (the talk's thesis), with **data-race freedom and a leaner trusted
base** as a security *bonus* — not as a "Rust is more secure than Kotlin" claim,
which the evidence doesn't support for typical backend services. The "70% fewer
vulnerabilities" stat belongs to the C/C++ comparison; using it against the JVM
would be misleading.

---

### Sources

- [Memory Errors and Memory Safety: A Look at Java and Rust — IEEE Security & Privacy (2023)](https://dl.acm.org/doi/abs/10.1109/MSEC.2023.3249719) · [companion PDF (Carleton)](https://people.scs.carleton.ca/~paulv/memSafety-for-2109.pdf)
- [Rust vs. Java: A Comprehensive Comparison of Memory-Safe Programming — Cogent University](https://www.cogentuniversity.com/post/rust-vs-java-a-comprehensive-comparison-of-memory-safe-programming-for-secure-high-performance-systems)
- [Memory Safety Is the Baseline: Rust, Safer C/C++, and JVM Reality in 2026 — Cogent University](https://www.cogentuniversity.com/post/memory-safety-is-the-baseline-rust-safer-c-c-and-jvm-reality-in-2026)
- [The Urgent Need for Memory Safety in Software Products — CISA](https://www.cisa.gov/news-events/news/urgent-need-memory-safety-software-products)
- [Rust Adoption Drives Android Memory Safety Bugs Below 20% for First Time — The Hacker News](https://thehackernews.com/2025/11/rust-adoption-drives-android-memory.html)
- [Google sees 68% drop in Android memory safety flaws over 5 years — BleepingComputer](https://www.bleepingcomputer.com/news/security/google-sees-68-percent-drop-in-android-memory-safety-flaws-over-5-years/)
- [Google says Android runs better when covered in Rust — The Register](https://www.theregister.com/2022/12/02/android_google_rust/)
- [Fearless Concurrency — The Rust Programming Language Book](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Extensible Concurrency with Send and Sync — The Rust Programming Language Book](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html)
- [Rust Safety: Writing Secure Concurrency without Fear — HackerOne](https://www.hackerone.com/blog/rust-safety-writing-secure-concurrency-without-fear)
- [Rust Won't Save Us: An Analysis of 2023's Known Exploited Vulnerabilities — Horizon3.ai](https://horizon3.ai/attack-research/attack-blogs/analysis-of-2023s-known-exploited-vulnerabilities/)
- [Log4Shell — Wikipedia (CVE-2021-44228)](https://en.wikipedia.org/wiki/Log4Shell)
- [Log4j RCE / Log4Shell explained — Snyk](https://snyk.io/blog/log4j-rce-log4shell-vulnerability-cve-2021-44228/)
- [RustSec Advisory Database](https://rustsec.org/) · [cargo-audit](https://crates.io/crates/cargo-audit) · [Security advisory for Cargo (CVE-2026-33056) — Rust Blog](https://blog.rust-lang.org/2026/03/21/cve-2026-33056/)

---

## AI coding agents on Rust in SWE-bench-style benchmarks

Research compiled 2026-06-13 via a multi-source, adversarially-verified deep-research
pass (18 of 25 fact-checked claims confirmed; 7 refuted and excluded). Supporting
material for the talk *"Introduction to Rust: Efficiency Beyond the JVM."*

### TL;DR

AI agents resolve only a **small fraction** of Rust software-engineering tasks, and
Rust sits among the **harder** languages in polyglot evaluations. The single true
"SWE-bench-style" benchmark (GitHub issue resolution) that includes Rust is
**Multi-SWE-bench** (ByteDance), where Claude-3.7-Sonnet resolves **5.4%–15.9%** of
Rust issues depending on the agent scaffold. Most other strong Rust evidence comes
from *narrower* tasks (C→Rust transpilation, API-evolution, compile-error repair) that
are **not** end-to-end issue resolution and are not directly comparable.

### 1. Which benchmarks actually cover Rust

- **Multi-SWE-bench** (ByteDance Seed, Apr 2025) — *the* SWE-bench-style polyglot
  benchmark with Rust. 1,632 human-validated GitHub issues across Java, TypeScript,
  JavaScript, Go, **Rust**, C, C++ (Python deliberately excluded; it is the non-Python
  complement to SWE-bench). [arXiv:2504.02605]
- **SWE-PolyBench** (Amazon Science, Apr 2025) — **omits Rust entirely**. Only
  Java/JS/TS/Python (2,110 instances). Useful only as a general "agents are modest
  across languages" baseline, **not** as Rust evidence. [arXiv:2504.08703]
- **SWE-bench Multilingual** / **SWE-rebench** — were in scope, but **no Rust-specific
  resolve rate survived verification**. Worth checking the live leaderboard directly
  (see open questions).

### 2. How well agents do on Rust (resolve rates)

On **Multi-SWE-bench**, Claude-3.7-Sonnet on the Rust subset [arXiv:2504.02605, Tables 4–5]:

| Agent scaffold | Rust resolved |
|---|---|
| Agentless | **5.44%** |
| SWE-agent | **6.69%** |
| OpenHands | **15.90%** |

- **Scaffold matters ~3×** — the agentic OpenHands loop roughly triples the Agentless baseline.
- By difficulty: Easy 10.6–21.2%, Medium 3–13%, **Hard 0.0%** across all three scaffolds.

### 3. Rust vs other languages

The paper groups languages into four domains: high-level general-purpose (Python,
Java), web (TS, JS), **systems (Go, Rust)**, low-level (C, C++). Verbatim: *"Go and
Rust… generally outperform TS and JS but fall behind Java."* For scale, in the same
benchmark Python resolves ~44.6% and Java ~14.1%, far above the Rust figures.
[arXiv:2504.02605]

> ⚠️ This Rust-vs-others ranking is **qualitative** and the authors call it
> *"inconsistent across models"* — treat as directional, not a precise leaderboard.

For a same-methodology cross-language baseline (no Rust), **SWE-PolyBench**'s best
agent (Aider-PB + Sonnet 3.5) scored 14.1% overall, with Python easiest at 20–24%,
Java 11–16%, TypeScript 5–13%. [arXiv:2504.08703] — illustrates how modest agent
resolve rates are in general, but **contains no Rust column**.

### 4. Why Rust is hard — and the compiler-feedback upside

- **Why hard:** the Multi-SWE-bench authors attribute systems-language difficulty
  (Rust grouped with C/C++/Go) to *"manual memory management, complex build systems,
  and intricate type systems."* [arXiv:2504.02605 §6.1.1]
- **The flip side (Rust's superpower for agents):** Rust's rich compiler errors enable
  effective **iterative repair loops**:
  - **RustAssistant** (Microsoft Research, ICSE 2025) — ~**74% peak** accuracy fixing
    real-world Rust compile errors via an LLM↔compiler loop (~93% on micro-benchmarks;
    GPT-4 > GPT-3.5). [arXiv:2308.05177]
  - **CRUST-Bench** (COLM 2025) — OpenAI o3 rises from **19% single-shot → 31%**
    (compiler-guided repair) **→ 48%** (compiler+test repair). Note test repair can
    *reduce* build success 5–20% by encouraging aggressive changes that introduce new
    borrow/type errors. [arXiv:2504.15254]
  - **SafeTrans** (RECODE 2026) — raw compiler messages alone are **insufficient**;
    *guided* repair lifts GPT-4o 54%→80%, DeepSeek-V3 49%→79% (borrow-checker
    violations resolved 74.2%). [arXiv:2505.10708]
- ⚠️ The **"less training data"** hypothesis is **not well-supported** by a verified
  source here — the papers attribute difficulty to memory/build/type-system
  complexity, not data scarcity.

### 5. Rust-specific (non-SWE-bench) evals worth citing

- **RustEvo²** (Mar 2025) — 588 Rust API changes (std + 15 crates, v1.71→1.84).
  Claude-3.7-Sonnet led Pass@1 at **65.3%** > o1-mini 57.5% > GPT-4o 55.4% >
  Gemini-1.5-Pro 55.3% > DeepSeek-v3 54.8%. (Narrow: API evolution.) [arXiv:2503.16922]
- **CRUST-Bench** (COLM 2025) — 100 C repos → safe Rust (avg ~958 LOC; 3,085 interface
  functions). Single-shot test-pass is low even for frontier models: Claude Opus 4
  **22%**, o3 19%, o1 15%, Claude 3.7 13%, Claude 3.5 11%, GPT-4o 7%, Gemini 1.5 Pro 3%.
  [arXiv:2504.15254]

### Caveats (read before quoting any number)

1. **Scope / terminology.** Only Multi-SWE-bench measures Rust *issue resolution*.
   CRUST-Bench / RustEvo² / RustAssistant / SafeTrans are transpilation /
   API-evolution / compile-repair — **do not present their % side-by-side with
   SWE-bench %**; they measure different things.
2. **Freshness.** Figures are 2024–2025-era models (Claude 3.5/3.7, Opus 4, GPT-4o,
   o1/o3, Gemini 1.5, DeepSeek-v3). RustAssistant's 74% is a 2023 GPT-4-era *peak*
   (best-case), not an average. These snapshots age quickly.
3. **Refuted — do not cite.** A specific Rust-subset size (239 instances / 10 repos),
   "borrow-checker is the single dominant failure cause" with specific %s, and several
   GPT-5-era Rust numbers (from arXiv:2602.21681) all **failed verification**.

### Sources (primary, behind verified findings)

| Source | Date | URL |
|---|---|---|
| Multi-SWE-bench (ByteDance) | Apr 2025 | https://arxiv.org/abs/2504.02605 |
| SWE-PolyBench (Amazon) — no Rust | Apr 2025 | https://arxiv.org/abs/2504.08703 |
| RustEvo² | Mar 2025 | https://arxiv.org/abs/2503.16922 |
| CRUST-Bench (UT Austin, COLM 2025) | 2025 | https://arxiv.org/html/2504.15254v3 |
| RustAssistant (Microsoft Research, ICSE 2025) | 2023/2025 | https://arxiv.org/abs/2308.05177 |
| SafeTrans (RECODE 2026) | 2026 | https://arxiv.org/html/2505.10708v2 |
| Multi-SWE-bench repo | 2025 | https://github.com/multi-swe-bench/multi-swe-bench |
| Multi-SWE-bench dataset | 2025 | https://huggingface.co/datasets/ByteDance-Seed/Multi-SWE-bench |
| RustEvo² repo | 2025 | https://github.com/SYSUSELab/RustEvo |
| SWE-bench Multilingual (check Rust live) | — | https://www.swebench.com/multilingual.html |
| Blog: "Rust and LLMs — the compiler does what code review shouldn't have to" | — | https://dev.to/arezvov/rust-and-llms-the-compiler-does-what-code-review-shouldnt-have-to-3ia4 |

### Open questions (not verified — check manually before the talk)

1. **Newer models' Rust resolve rates** on Multi-SWE-bench — verified data is
   Claude-3.7-centric; no public *Rust* leaderboard surfaced for 2025–2026 frontier
   models.
2. **Size of the Rust subset** of Multi-SWE-bench (the 239/10 figure was refuted).
3. Whether **SWE-bench Multilingual / SWE-rebench** include Rust, and their rates.
4. Direct empirical evidence (vs qualitative attribution) isolating *why* Rust is
   harder — controlled studies separating training-data volume from
   borrow-checker/lifetime/type-system difficulty.
