# Resources

## General Rust performance benchmarks
- [TechEmpower Web Framework Benchmarks](https://www.techempower.com/benchmarks/) — independent, third-party benchmarks. Rust frameworks (actix, axum) routinely sit at the top; Spring is far down the list.
  - **Round 23 provenance (for the slide table):** the project was archived in 2025 and the data backend (`tfb-status.techempower.com`) is offline, so the site no longer renders results. The R23 Fortunes top-14 numbers on the slide come from screenshots of the official chart in [tokio-rs/axum discussion #3266](https://github.com/tokio-rs/axum/discussions/3266) (may-minihttp 1,327,378 req/s at #1; first JVM entries vert.x #13 and Quarkus #14 at ~78% of the leader). Run metadata (841 implementations, completed 2025-02-05) per the [archived run page](https://web.archive.org/web/20251209025800/https://tfb-status.techempower.com/results/91a66052-9d86-446c-b31a-eadbd669ed08). Note: actix variants failed to build in R23 (pinned to Rust 1.58 from 2022, per [maintainer comment](https://github.com/TechEmpower/FrameworkBenchmarks/issues/9589)), hence absent from that round's top list. The Spring fortunes figure on the slide (243,639 req/s, ~18% of #1) is **[secondary]** from [tuananhpham's R23 analysis](https://dev.to/tuananhpham/popular-backend-frameworks-performance-benchmark-1bkh); Spring's exact global rank is not recoverable. The article does not name the exact variant — most likely the non-reactive `spring` entry (Spring Boot MVC, servlet, full ORM); R23 also ran `spring-webflux` (Netty, R2DBC). Context on the archiving: [TechEmpower Benchmarks are now archived — what's next?](https://dev.to/kaliumhexacyanoferrat/techempower-framework-benchmarks-are-now-archived-whats-next-3l0a)
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
- https://www.reddit.com/r/programming/s/rlmz18Vbcg
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

### 2026 follow-ups (the before/after timeline slide)
- [Revisiting Rust in 2026 — Matt Welsh](https://mdwdotla.medium.com/revisiting-rust-in-2026-ae8720cc7f2c) — the author of the 2022 "cautionary tale" (below) revisits it: LLMs flattened the learning curve ("ask all the dumb questions", tens of thousands of lines, far more productive). Caveats intact: hiring pool still small (JS/Python applicants outnumber Rust by orders of magnitude), and AI Rust can be non-idiomatic "slop" — he restricts juniors from AI codegen on code they can't read.
- [Why Rust Wins in the Age of AI — HAMY](https://hamy.xyz/blog/2026-04_rust-age-of-ai) — SWE-bench Multilingual: agents complete **58.14%** of Rust tasks, best of 9 languages (iterative/multi-shot). Counterweight: one-shot generation favors Ruby/Python by 1.4–2.6x in speed/cost.
- [Why Rust Is Winning for AI Tooling in 2026 — dasroot](https://dasroot.net/posts/2026/02/why-rust-winning-ai-tooling-2026/) — AI assistance plus ecosystem maturity is making Rust more accessible.
- [RustCoder: AI-assisted Rust learning — CNCF](https://www.cncf.io/blog/2025/01/10/rustcoder-ai-assisted-rust-learning/) — onboarding angle: AI tutor reduced onboarding time for 1,000+ students (OpenAtom, Tsinghua coding camp).
- [Programming Languages and Type Safety in the Era of LLMs — The Coded Message](https://www.thecodedmessage.com/posts/pls-in-llm-land/) — the type-safety-as-agent-guardrail argument from a language-design angle.

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
