# gg performance baseline

gg is a process, not a library, and everything it costs is paid per
spawn. Two paths matter:

- **`gg -i zsh`** runs on every interactive shell start. On this
  owner's machine `~/.zsh/tools.zsh` runs it **four times**, so it sits
  on the critical path of opening a terminal.
- **`gg --get`** runs once per `gg <path>` command, before the shell
  function runs `git clone` or `git fetch`.

`benches/bench.rs` spawns the real binary and measures both end to end,
dyld and clap included. Measuring the URL regex in isolation would be
measuring the cheapest part.

## Running them

```bash
cargo bench --bench bench                     # run them
cargo bench --bench bench -- --lock           # re-lock the baseline
cargo bench --bench bench -- --gate           # compare to the baseline
cargo bench --bench bench -- --gate --loose   # counted metrics only
```

The gate fails in **both** directions. A regression needs a fix; an
improvement needs a re-lock in the same commit as the change that
earned it, because a baseline nobody re-locks stops describing the code
and the next regression only has to stay under the stale number.

## Timings are floors, and there is a control

Each benchmark runs many timed rounds and reports the **fastest**. Load
on the machine can only ever make a round slower, so the floor is the
closest thing to an uncontended measurement.

`SpawnControl` spawns `/usr/bin/true`. It is there so the other two can
be read as "gg, plus what a spawn costs on this machine" rather than as
absolute numbers — without it, a change in process-creation cost is
indistinguishable from a change in gg.

## Reference machine

| | |
|---|---|
| Machine | Apple M4 Max, macOS 26.6.2, arm64 |
| Toolchain | stable, edition 2021 |
| Recorded | 2026-09-07 |

## Locked numbers

| Benchmark | ns/op floor | gg's own share |
|---|---|---|
| `SpawnControl` (84 KB binary, does nothing) | 1.22 ms | — |
| `InstallZsh` | 2.13 ms | ~0.9 ms |
| `GetExistingRepo` | 2.46 ms | ~1.2 ms |

Counted exactly: 1,075 bytes of shell emitted by `-i zsh` and 190 bytes
by `--get`. The binary's own size is reported but never gated — a
different toolchain on a different platform produces a different size,
and gating on that would fail every CI run while telling nobody
anything. CI runs `--gate --loose`, which compares the two byte counts
and nothing else.

## What was tried and did not work

**Shrinking the binary does not make it start faster.** The obvious
hypothesis was that dyld maps and relocates 2.2 MB on every one of
those four shell-start invocations. Thin LTO, one codegen unit and
`strip` took the binary from 2,175,712 to 1,416,304 bytes — 35%
smaller — and moved the measured floor by nothing at all (1.949 ms to
1.954 ms on `InstallZsh`, inside the noise, and `GetExistingRepo`
slightly worse). So no such profile ships: a `panic = "abort"`
behaviour change bought no measured latency.

More than half of each invocation is process creation the program
cannot influence — the control says 1.22 ms of it. gg's own share is
under a millisecond, and no hot path was found inside it: the URL match
is one regex over a short string, and the rest is a handful of `stat`
calls.

The value of these benchmarks is therefore **regression protection**
rather than a starting point for optimisation. A dependency that adds
startup work, or a change that does I/O before deciding what to do,
shows up here as a floor that moved — measured against a control that
did not.
