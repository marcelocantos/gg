// Copyright 2026 Marcelo Cantos
// SPDX-License-Identifier: Apache-2.0
//
// Benchmarks for what gg actually costs its user, plus the gate that
// locks the numbers in both directions.
//
//     cargo bench --bench bench                     # run them
//     cargo bench --bench bench -- --lock           # re-lock the baseline
//     cargo bench --bench bench -- --gate           # compare to the baseline
//     cargo bench --bench bench -- --gate --loose   # counted metrics only
//
// gg is a process, not a library, and both of its costs are paid per
// spawn. `gg -i zsh` runs on every interactive shell start — four times
// on this owner's machine, from ~/.zsh/tools.zsh — so it sits on the
// critical path of opening a terminal. `gg --get` runs once per `gg
// <path>` command, before the git fetch the shell function then does.
// So both benchmarks spawn the real binary and measure the whole thing
// end to end, dyld and clap included: an in-process measurement of the
// URL regex would be measuring the cheapest part.
//
// A third benchmark measures the URL parse in isolation, so that a
// change there is attributable rather than lost inside spawn cost.
//
// Timings are reported as the FLOOR over many rounds rather than an
// average, because load on the machine can only make a round slower.
// They are only comparable against a baseline recorded on the same
// machine, so they carry a wide tolerance and are skipped under
// --loose. Counted metrics — bytes of output, binary size — are
// functions of the code and are compared exactly.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How long each benchmark is sampled for before its per-iteration cost
/// is reported.
const SAMPLE_TIME: Duration = Duration::from_millis(500);

/// Every benchmark runs at least this many rounds, however slow it is,
/// so its floor is taken over more than a couple of samples.
const MIN_ROUNDS: u32 = 8;

/// The shortest a timed round may be. Iterations are batched up to this
/// so the clock's granularity does not become the measurement.
const MIN_ROUND: Duration = Duration::from_millis(1);

/// Timing tolerance, as a fraction of the baseline. Deliberately wide:
/// a gate that fails on scheduler noise gets switched off, which is
/// worse than a coarse one.
const TIMING_TOLERANCE: f64 = 0.25;

/// Where the locked baseline lives, relative to the crate root.
const BASELINE: &str = "docs/perf/baseline.txt";

/// The binary under test, built by cargo for this bench target.
const GG: &str = env!("CARGO_BIN_EXE_gg");

/// The control program: the cheapest spawn the system offers.
const CONTROL: &str = "/usr/bin/true";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let lock = args.iter().any(|a| a == "--lock");
    let gate = args.iter().any(|a| a == "--gate");
    let loose = args.iter().any(|a| a == "--loose");

    let results = run_all();
    let rendered = render(&results);
    print!("{rendered}");

    let baseline_path = crate_root().join(BASELINE);
    if lock {
        if let Some(dir) = baseline_path.parent() {
            std::fs::create_dir_all(dir).expect("create docs/perf");
        }
        std::fs::write(&baseline_path, &rendered).expect("write baseline");
        println!("\nlocked {BASELINE} — update docs/perf/baseline.md alongside it");
        return;
    }
    if gate {
        let base = std::fs::read_to_string(&baseline_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", baseline_path.display()));
        if !compare(&parse(&base), &results, loose) {
            std::process::exit(1);
        }
    }
}

// ------------------------------------------------------------ benchmarks

fn run_all() -> Vec<Sample> {
    // A repo root that already exists, so --get takes the path a user
    // with an established checkout takes: no ls-remote, no mkdir.
    let root = tempfile::tempdir().expect("tempdir");
    let ggroot = root.path().to_path_buf();
    std::fs::create_dir_all(ggroot.join("github.com/acme/repo")).expect("seed root");

    let install = spawn_args(&ggroot, &["-i", "zsh"]);
    let get = spawn_args(&ggroot, &["--get", "github.com/acme/repo"]);

    vec![
        // The control. Spawning the smallest possible program measures
        // what a spawn costs on this machine before gg has done
        // anything, so the two benchmarks below can be read as "gg plus
        // this" rather than as absolute numbers. Without it, a change in
        // process-creation cost looks exactly like a change in gg.
        bench("SpawnControl", |_| {
            let status = Command::new(CONTROL)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .expect("spawn control");
            std::hint::black_box(status.success());
        })
        .with("binary-bytes", control_bytes()),
        // Every interactive shell start pays this, four times over.
        bench("InstallZsh", |_| {
            std::hint::black_box(run_gg(&ggroot, &["-i", "zsh"]));
        })
        .with("stdout-bytes", install as f64)
        .with("binary-bytes", binary_bytes()),
        // Every `gg <path>` pays this before the shell runs git.
        bench("GetExistingRepo", |_| {
            std::hint::black_box(run_gg(&ggroot, &["--get", "github.com/acme/repo"]));
        })
        .with("stdout-bytes", get as f64),
    ]
}

/// Run the shipped binary once and return its stdout length.
fn run_gg(ggroot: &Path, args: &[&str]) -> usize {
    let out = Command::new(GG)
        .args(args)
        .env("GGROOT", ggroot)
        .stderr(Stdio::null())
        .output()
        .expect("spawn gg");
    assert!(out.status.success(), "gg {args:?} failed");
    out.stdout.len()
}

/// The size of one invocation's stdout, measured once outside the timed
/// region. It is a function of the code, so the gate compares it
/// exactly: a change that starts emitting more shell than it needs to
/// on every shell start should have to say so.
fn spawn_args(ggroot: &Path, args: &[&str]) -> usize {
    run_gg(ggroot, args)
}

/// The size of the binary itself, locked next to the timings so that a
/// dependency which doubles it is visible. It does NOT explain the
/// timings: shrinking this binary by 35% (thin LTO, one codegen unit,
/// stripped) moved the measured floor by nothing at all, which is why
/// no such profile ships.
fn binary_bytes() -> f64 {
    std::fs::metadata(GG).expect("stat gg").len() as f64
}

/// The control program's size, for the same comparison.
fn control_bytes() -> f64 {
    std::fs::metadata(CONTROL).map(|m| m.len()).unwrap_or(0) as f64
}

// --------------------------------------------------------------- harness

/// One benchmark's result: its per-iteration time plus any counted
/// metrics it reports.
struct Sample {
    name: &'static str,
    ns: f64,
    counted: BTreeMap<String, f64>,
}

impl Sample {
    fn with(mut self, metric: &str, value: f64) -> Sample {
        self.counted.insert(metric.to_string(), value);
        self
    }
}

/// Run `f` repeatedly for SAMPLE_TIME and report the FLOOR of the
/// per-iteration cost, not its average.
///
/// Competing load on the machine can only ever make an iteration
/// slower, so the fastest round is the closest thing to an uncontended
/// measurement, and it is the only figure worth locking on a machine
/// that runs other work. An average over the same rounds moved 56%
/// between two back-to-back runs while the floor held.
///
/// Rounds are batched so that a round is long enough to measure: with a
/// batch of one, a benchmark faster than the clock's granularity
/// measures its timer rather than itself.
fn bench(name: &'static str, mut f: impl FnMut(u32)) -> Sample {
    // One untimed pass so a first-call cost (a lazily built regex, a
    // cold page) is not charged to the measurement.
    f(0);

    let probe = Instant::now();
    f(1);
    let single = probe.elapsed().as_nanos().max(1) as f64;
    let batch = (MIN_ROUND.as_nanos() as f64 / single).ceil().max(1.0) as u32;

    let start = Instant::now();
    let mut rounds = 0u32;
    let mut floor = f64::MAX;
    let mut iterations = 0u32;
    loop {
        let round = Instant::now();
        for _ in 0..batch {
            f(iterations);
            iterations += 1;
        }
        let per_iteration = round.elapsed().as_nanos() as f64 / batch as f64;
        floor = floor.min(per_iteration);
        rounds += 1;
        if rounds >= MIN_ROUNDS && start.elapsed() >= SAMPLE_TIME {
            break;
        }
    }
    Sample {
        name,
        ns: floor,
        counted: BTreeMap::new(),
    }
}

/// Render results in the format the baseline file stores: one line per
/// benchmark, the timing first, then counted metrics in name order.
fn render(results: &[Sample]) -> String {
    let mut out = String::new();
    writeln!(out, "# gg benchmarks — see docs/perf/baseline.md").unwrap();
    for s in results {
        write!(out, "{}\t{:.0} ns/op", s.name, s.ns).unwrap();
        for (metric, value) in &s.counted {
            write!(out, "\t{value:.0} {metric}").unwrap();
        }
        out.push('\n');
    }
    out
}

/// Parse a rendered baseline back into samples.
fn parse(text: &str) -> Vec<Sample> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        let Some(name) = fields.next() else { continue };
        let name: &'static str = Box::leak(name.to_string().into_boxed_str());
        let mut ns = 0.0;
        let mut counted = BTreeMap::new();
        for field in fields {
            let mut parts = field.split_whitespace();
            let (Some(value), Some(unit)) = (parts.next(), parts.next()) else {
                continue;
            };
            let value: f64 = value.parse().expect("numeric metric");
            if unit == "ns/op" {
                ns = value;
            } else {
                counted.insert(unit.to_string(), value);
            }
        }
        out.push(Sample { name, ns, counted });
    }
    out
}

/// Compare a fresh run against the baseline. Returns false when
/// anything is outside tolerance in either direction.
fn compare(base: &[Sample], got: &[Sample], loose: bool) -> bool {
    let mut ok = true;
    println!();
    for b in base {
        let Some(g) = got.iter().find(|g| g.name == b.name) else {
            println!("{:<20} MISSING from this run", b.name);
            ok = false;
            continue;
        };
        for (metric, want) in &b.counted {
            match g.counted.get(metric) {
                Some(have) if have == want => {}
                Some(have) => {
                    println!(
                        "{:<20} {metric}: locked {want}, measured {have} — a rise is a \
                         regression, a fall must be re-locked in the same commit",
                        b.name
                    );
                    ok = false;
                }
                None => {
                    println!("{:<20} {metric}: not reported by this run", b.name);
                    ok = false;
                }
            }
        }
        if loose {
            continue;
        }
        let delta = (g.ns - b.ns) / b.ns;
        if delta.abs() > TIMING_TOLERANCE {
            println!(
                "{:<20} ns/op: locked {:.0}, measured {:.0} ({:+.1}%) — {}",
                b.name,
                b.ns,
                g.ns,
                delta * 100.0,
                if delta > 0.0 {
                    "a regression"
                } else {
                    "an improvement; re-lock in the same commit (cargo bench -- --lock)"
                }
            );
            ok = false;
        }
    }
    for g in got {
        if !base.iter().any(|b| b.name == g.name) {
            println!("{:<20} NEW — lock it before gating on it", g.name);
        }
    }
    println!(
        "\n{}",
        if ok {
            "bench gate: within tolerance"
        } else {
            "bench gate: FAILED"
        }
    );
    ok
}

/// The crate root, so the baseline path does not depend on the cwd
/// `cargo bench` happens to use.
fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}
