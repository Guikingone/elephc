//! Purpose:
//! Profiles a program `monitor` launches — a `.php` source (built first) or a
//! binary. This path records exact instrumented calls, wall time, allocations,
//! retained objects, database queries, and database-driver wait; it does not
//! claim operating-system CPU time or file-I/O events.
//!
//! Called from:
//! - `monitor::main`, when the target is a file rather than an address.
//!
//! Key details:
//! - A binary without the capability is refused, not profiled approximately.
//! - The control channel is established before the spawn; the program reads it
//!   during its own init.

use super::*;

/// One sampling window over the whole process tree, rendered once: the bar
/// table on stdout, the Speedscope file, and the CI summary when applicable.
pub(crate) fn run_once(
    cmd: &MonitorCommand,
    root: u32,
    binary: Option<&Path>,
    php_source: Option<&Path>,
) -> i32 {
    let pids = discover_pids(root);
    let reports = capture_window(&pids, cmd.duration_secs);
    let samples = match samples_from_reports(&reports, binary, php_source) {
        Some(samples) => samples,
        None => {
            eprintln!(
                "elephc monitor: no samples captured — the program may have exited before \
                 sampling started; try a longer-running input"
            );
            return 1;
        }
    };
    let display = render_stacks(&samples);
    let out_path = cmd
        .out
        .clone()
        .unwrap_or_else(|| format!("{}.speedscope.json", cmd.target.trim_end_matches(".php")));
    if !cmd.target.is_empty() || cmd.out.is_some() {
        match write_speedscope(&display, &out_path) {
            Ok(()) => println!("wrote {out_path}"),
            Err(error) => {
                eprintln!("elephc monitor: {error}");
                return 1;
            }
        }
    }
    print!("{}", why_table(&display, pids.len()));
    write_github_summary(&display, pids.len());
    if let Some(pprof_path) = &cmd.pprof_out {
        let stacks = php_folded_stacks(&display);
        let encoded = crate::pprof_encode::encode_folded_profile(&stacks);
        match std::fs::write(pprof_path, encoded) {
            Ok(()) => println!("wrote {pprof_path}"),
            Err(error) => {
                eprintln!("elephc monitor: cannot write {pprof_path}: {error}");
                return 1;
            }
        }
    }
    let graph_title = if cmd.target.is_empty() {
        "elephc profile".to_string()
    } else {
        cmd.target.trim_end_matches(".php").to_string()
    };
    // Per-line attribution needs the dSYM and the source, so it rides the same
    // .php-target path that recovers inlined frames.
    let lines = match (binary, php_source) {
        (Some(binary), Some(source)) => reports
            .iter()
            .find_map(|report| line_profile(&samples, report, binary, source)),
        _ => None,
    };
    if let Err(error) = write_graph_exports(cmd, &display, &graph_title, lines.as_ref()) {
        eprintln!("elephc monitor: {error}");
        return 1;
    }
    if let Some(baseline_path) = &cmd.baseline {
        match diff_against_baseline(&display, baseline_path, cmd.fail_on_regression) {
            Ok(code) => return code,
            Err(error) => {
                eprintln!("elephc monitor: {error}");
                return 1;
            }
        }
    }
    0
}

/// The live loop: sample a window, merge the process tree, redraw, repeat
/// until the target goes away. Prints the cumulative table on exit.
pub(crate) fn run_live(cmd: &MonitorCommand, root: u32, mut child: Option<&mut process::Child>) -> i32 {
    use std::io::IsTerminal;
    let interactive = std::io::stdout().is_terminal();
    let started = std::time::Instant::now();
    let mut cumulative: BTreeMap<Vec<(String, Kind)>, u64> = BTreeMap::new();
    let mut previous: HashMap<String, f64> = HashMap::new();
    let mut windows = 0u32;
    let graph_title = if cmd.target.is_empty() {
        "elephc profile".to_string()
    } else {
        cmd.target.trim_end_matches(".php").to_string()
    };
    // Rolling window of the last 10 per-window call graphs for the live HTML.
    let mut html_ring: std::collections::VecDeque<(u128, crate::call_graph::CallGraph)> =
        std::collections::VecDeque::new();
    if let (Some(addr), Some(path)) = (&cmd.serve, &cmd.html_out) {
        match serve_live_file(addr, path.clone()) {
            Ok(local) => eprintln!(
                "elephc monitor: serving live call graph at http://{local}/ (updates in place)"
            ),
            Err(error) => eprintln!("elephc monitor: cannot serve on {addr}: {error}"),
        }
    }
    loop {
        if let Some(child) = child.as_deref_mut() {
            if child.try_wait().ok().flatten().is_some() {
                break;
            }
        }
        let pids = discover_pids(root);
        let reports = capture_window(&pids, cmd.duration_secs);
        let Some(samples) = samples_from_reports(&reports, None, None) else {
            // Attach mode has no child handle: a window with zero reports is
            // how we learn the target is gone.
            break;
        };
        windows += 1;
        let display = render_stacks(&samples);
        for (stack, weight) in &display {
            *cumulative.entry(stack.clone()).or_default() += weight;
        }
        if cmd.html_out.is_some() || cmd.dot_out.is_some() {
            write_live_graphs(cmd, &display, &graph_title, &mut html_ring);
        }
        let frame = live_frame(
            &display,
            &cumulative,
            &mut previous,
            pids.len(),
            cmd.duration_secs,
            started.elapsed(),
        );
        if interactive {
            // Clear and home, like top: the frame replaces the previous one.
            print!("\u{1b}[2J\u{1b}[H{frame}");
            let _ = std::io::stdout().flush();
        } else {
            println!("--- window {windows} ---");
            print!("{frame}");
        }
    }
    if windows > 0 {
        let merged: Vec<(Vec<(String, Kind)>, u64)> = cumulative.into_iter().collect();
        println!("\n=== cumulative ({windows} windows) ===");
        print!("{}", why_table(&merged, 1));
    }
    0
}

/// Samples every pid of one window in parallel and returns the reports that
/// succeeded — a worker dying mid-window degrades coverage, never the run.
pub(crate) fn capture_window(pids: &[u32], duration_secs: u32) -> Vec<String> {
    let mut jobs = Vec::new();
    for pid in pids {
        let report_path = std::env::temp_dir().join(format!(
            "elephc_monitor_{}_{}.txt",
            process::id(),
            pid
        ));
        let child = process::Command::new("/usr/bin/sample")
            .args([
                pid.to_string(),
                duration_secs.to_string(),
                "-file".to_string(),
                report_path.display().to_string(),
            ])
            .stdout(process::Stdio::null())
            // Kept, not discarded: when the sampler refuses, its own sentence is
            // the only thing that says why, and a caller that sees "no samples"
            // cannot reconstruct it.
            .stderr(process::Stdio::piped())
            .spawn();
        if let Ok(child) = child {
            jobs.push((child, report_path));
        }
    }
    let mut reports = Vec::new();
    let mut refusal = None;
    for (job, report_path) in jobs {
        let done = job.wait_with_output();
        let ok = done.as_ref().map(|o| o.status.success()).unwrap_or(false);
        if ok {
            if let Ok(text) = std::fs::read_to_string(&report_path) {
                reports.push(text);
            }
        } else if refusal.is_none() {
            refusal = done
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stderr).trim().to_string())
                .filter(|s| !s.is_empty());
        }
        let _ = std::fs::remove_file(&report_path);
    }
    // Say it once, not once per window: a --live loop would otherwise bury the
    // table under the same line every few seconds.
    if reports.is_empty() {
        if let Some(message) = refusal {
            static SAID: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !SAID.swap(true, std::sync::atomic::Ordering::Relaxed) {
                let first = message.lines().next().unwrap_or(&message);
                eprintln!("elephc monitor: the sampler refused: {first}");
            }
        }
    }
    reports
}

/// Makes a compiled program's path safe to hand to `Command::new`.
///
/// `Command::new("shop")` does not run `./shop`: with no separator in the name the
/// OS searches `PATH`, so spawning fails with `No such file or directory` even
/// though the binary is right there. It appears to work on a machine whose `PATH`
/// carries an empty entry — POSIX reads that as the current directory — which is
/// exactly the kind of accident that hides the bug during development and surfaces
/// it for everyone else. Absolute wins: unambiguous, and it survives any later
/// change of working directory.
/// A name that resolves to no local file is left alone, so `monitor some-tool` can
/// still mean a program on `PATH`.
pub(crate) fn spawnable_path(path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() || !path.exists() {
        return path;
    }
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(path),
        // Without a cwd there is nothing better than an explicit relative path,
        // which at least defeats the PATH search.
        Err(_) => PathBuf::from(".").join(path),
    }
}

/// Compiles a `.php` target with `--debug-info` by re-executing this binary, and
/// returns the produced executable's path (next to the source, like a normal compile).
pub(crate) fn compile_php_target(source: &str) -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate elephc: {e}"))?;
    let status = process::Command::new(exe)
        .args(["--debug-info", source])
        .status()
        .map_err(|e| format!("cannot run elephc: {e}"))?;
    if !status.success() {
        return Err(format!("compiling {source} failed"));
    }
    Ok(spawnable_path(source.trim_end_matches(".php")))
}

/// Explains an empty capture by what the run actually did.
///
/// One message covered every cause: "was the target built with
/// --with-monitoring". For a program that crashed after printing its output —
/// which is what a CI shard showed, on one architecture only — that is a
/// confident diagnosis of the wrong thing, and it sent the investigation at the
/// build flags for an hour. The exit status was there the whole time and nobody
/// looked at it.
pub(crate) fn no_profile_reason(
    status: &process::ExitStatus,
    binary: &Path,
    capture_activated: bool,
) -> String {
    use std::os::unix::process::ExitStatusExt as _;
    if let Some(signal) = status.signal() {
        return format!(
            "{} was killed by signal {signal} before the active capture window could close \
             and publish its profile",
            binary.display()
        );
    }
    match status.code() {
        Some(code) if code != 0 => format!(
            "{} exited with status {code} before the active capture window could close and \
             publish its profile",
            binary.display()
        ),
        _ if !capture_activated => format!(
            "the exact control channel for {} was unavailable or was not acknowledged, so no \
             capture window was activated",
            binary.display()
        ),
        Some(0) | None => format!(
            "{} completed a valid capture window with no instrumented frames; under selective \
             instrumentation this means none of the selected functions ran",
            binary.display()
        ),
        Some(_) => unreachable!("non-zero statuses were handled above"),
    }
}

/// Reads a target that carries the monitoring capability exactly: run it, and
/// render the profile it prints to stderr — the deterministic counterpart to
/// sampling. Honors `--dot` / `--html`.
pub(crate) fn run_instrument(cmd: &MonitorCommand) -> i32 {
    let binary = if cmd.target.ends_with(".php") {
        match compile_php_monitored(&cmd.target) {
            Ok(path) => path,
            Err(error) => {
                eprintln!("elephc monitor: {error}");
                return 1;
            }
        }
    } else {
        let path = spawnable_path(&cmd.target);
        // Nothing to switch on if the hooks were never compiled in.
        if !carries_monitoring(&path) {
            eprintln!(
                "elephc monitor: {} carries no monitoring — rebuild it with \
                 --with-monitoring, or point monitor at the .php source",
                path.display()
            );
            return 1;
        }
        path
    };
    // Inherit stdout (the program's own output shows live); capture stderr, where
    // the instrument profile is written at exit.
    let mut command = process::Command::new(&binary);
    command.stderr(process::Stdio::piped());
    // The binary carries the hooks but boots dormant, so being asked is what
    // separates "capable" from "profiling". Asking happens over a socketpair only
    // this process holds the other end of, rather than an environment variable
    // every process on the machine can read.
    let channel = open_control_channel();
    if let Some(channel) = &channel {
        attach_control_channel(&mut command, channel);
    }
    if let Some(trace_path) = &cmd.trace {
        // The runtime writes the Chrome/Perfetto trace to this path at exit.
        command.env("ELEPHC_INSTR_TRACE", trace_path);
    }
    let output = match command
        .spawn()
        .and_then(|child| child.wait_with_output())
    {
        Ok(output) => output,
        Err(error) => {
            eprintln!("elephc monitor: cannot run {}: {error}", binary.display());
            return 1;
        }
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    let capture_activated = channel.as_ref().is_some_and(control_channel_activated);
    // Pass through the program's own diagnostics — and only those. A
    // `--with-monitoring` binary carries both mechanisms, so its stderr also
    // holds the sampler's folded stacks; forwarding those would print raw
    // profiler output as if the program had written it.
    for line in stderr.lines() {
        if !line.starts_with("elephc-instr") && !line.starts_with("elephc-probe") {
            eprintln!("{line}");
        }
    }
    // The runtime writes its warnings to the same stderr the parser consumes,
    // so anything the parser does not recognise would vanish. A truncated
    // profile that says nothing is worse than no profile: pass them through.
    for line in stderr.lines() {
        if let Some(note) = line.strip_prefix("elephc-instr: note: ") {
            eprintln!("elephc monitor: {note}");
        }
    }
    let mut graph = parse_instrument_dump(&stderr);
    if graph.nodes.is_empty() {
        eprintln!(
            "elephc monitor: {}",
            no_profile_reason(&output.status, &binary, capture_activated)
        );
        return 1;
    }
    print!("{}", instrument_table(&graph));
    let title = cmd.target.trim_end_matches(".php").to_string();
    // The exact capture carries no per-line data, but it can still show the
    // file: every measured function, located, with its cost.
    attach_exact_source(&mut graph, &cmd.target);
    // Assertions come from the project budget file and from --assert, in that
    // order: the file states the standing contract, the flag adds a one-off.
    // Evaluated here, before any export, so the page can carry the verdicts.
    let mut asserts: Vec<(String, Option<String>)> = Vec::new();
    match load_assert_file(cmd.assert_file.as_deref(), &cmd.target) {
        Ok(Some((from_file, path))) => {
            if !from_file.is_empty() {
                println!("assertions: {} from {path}", from_file.len());
            }
            asserts.extend(from_file);
        }
        Ok(None) => {}
        Err(error) => {
            eprintln!("elephc monitor: {error}");
            return 1;
        }
    }
    asserts.extend(cmd.asserts.iter().map(|spec| (spec.clone(), None)));
    let assert_outcomes = evaluate_asserts(&graph, &asserts);
    // The same exports the sampled path offers, from measured time rather than
    // from samples: a Speedscope document and a pprof profile.
    {
        let stacks = exact_stacks(&graph);
        // Same default as the sampled path, deliberately: "the same exports
        // whichever target produced them" is only true if the default is the
        // same one too.
        let out_path = cmd
            .out
            .clone()
            .unwrap_or_else(|| format!("{}.speedscope.json", cmd.target.trim_end_matches(".php")));
        if let Err(error) = write_speedscope(&stacks, &out_path) {
            eprintln!("elephc monitor: {error}");
            return 1;
        }
        println!("wrote {out_path}");
        if let Some(pprof_path) = &cmd.pprof_out {
            let folded = php_folded_stacks(&stacks);
            let encoded = crate::pprof_encode::encode_folded_profile(&folded);
            match std::fs::write(pprof_path, encoded) {
                Ok(()) => println!("wrote {pprof_path}"),
                Err(error) => {
                    eprintln!("elephc monitor: cannot write {pprof_path}: {error}");
                    return 1;
                }
            }
        }
    }
    // Save this capture for use as a later --baseline.
    if let Some(path) = &cmd.save {
        match serde_json::to_string(&graph) {
            Ok(json) => match std::fs::write(path, json) {
                Ok(()) => println!("saved exact capture to {path}"),
                Err(error) => {
                    eprintln!("elephc monitor: cannot save {path}: {error}");
                    return 1;
                }
            },
            Err(error) => {
                eprintln!("elephc monitor: cannot serialize capture: {error}");
                return 1;
            }
        }
    }
    // Load a prior exact capture to diff against.
    //
    // An unreadable baseline is an ERROR, not a warning. This warned and carried
    // on, so `--baseline` on a file it could not parse still exited 0 — and with
    // `--fail-on-regression` that is a CI gate reporting success for a comparison
    // it never made, which is the one thing a gate must never do.
    let mut baseline = None;
    if let Some(path) = &cmd.baseline {
        match load_exact_graph(path) {
            Some(graph) => baseline = Some(graph),
            None => {
                eprintln!("elephc monitor: could not read exact baseline {path}");
                // The likeliest cause by far, and the one the docs used to
                // suggest: a Speedscope export is a different document with
                // different data, not an exact capture.
                if looks_like_speedscope(path) {
                    eprintln!(
                        "  {path} is a Speedscope export, which carries no per-function \
                         measurements to compare against.\n  \
                         Produce the baseline with --save instead:  elephc monitor \
                         <target> --save baseline.json"
                    );
                }
                return 1;
            }
        }
    }
    if let Some(path) = &cmd.dot_out {
        if let Err(error) = std::fs::write(path, crate::call_graph::render_dot(&graph)) {
            eprintln!("elephc monitor: cannot write {path}: {error}");
            return 1;
        }
        println!("wrote {path}");
    }
    if let Some(path) = &cmd.html_out {
        // With a baseline, render two exact frames [baseline, current] so the
        // navigator scrubs between them and the diff mode highlights growth.
        let html = match &baseline {
            Some(base) => crate::call_graph::render_html_frames(
                &[(0, base), (1, &graph)],
                &title,
                false,
                0,
                true,
                &assert_outcomes,
            ),
            None => crate::call_graph::render_html_exact(&graph, &title, &assert_outcomes),
        };
        if let Err(error) = std::fs::write(path, html) {
            eprintln!("elephc monitor: cannot write {path}: {error}");
            return 1;
        }
        println!("wrote {path}");
    }
    if let Some(path) = &cmd.trace {
        if std::path::Path::new(path).exists() {
            println!("wrote {path} — open in https://ui.perfetto.dev or chrome://tracing");
        }
    }
    if let Some(base) = &baseline {
        print!("{}", instrument_delta_table(base, &graph));
    }
    print!("{}", instrument_recommendations(&graph));
    if !assert_outcomes.is_empty() {
        let (report, ok) = assert_report(&assert_outcomes);
        print!("{report}");
        if !ok {
            return 2;
        }
    }
    0
}

/// Renders one live frame: the window's hot functions with trend arrows
/// against the previous window, and the cumulative share on the right.
pub(crate) fn live_frame(
    window: &[(Vec<(String, Kind)>, u64)],
    cumulative: &BTreeMap<Vec<(String, Kind)>, u64>,
    previous: &mut HashMap<String, f64>,
    processes: usize,
    window_secs: u32,
    elapsed: std::time::Duration,
) -> String {
    let stats = table_stats(window);
    let cumulative_samples: Vec<(Vec<(String, Kind)>, u64)> =
        cumulative.iter().map(|(k, v)| (k.clone(), *v)).collect();
    let cumulative_stats = table_stats(&cumulative_samples);
    let elapsed_secs = elapsed.as_secs();
    let mut out = format!(
        "elephc monitor — live · {processes} process{} · window {window_secs}s · total {}m{:02}s · {} samples\n",
        if processes > 1 { "es" } else { "" },
        elapsed_secs / 60,
        elapsed_secs % 60,
        cumulative_stats.grand,
    );
    out.push_str(&format!(
        "{:<26} {:<22} {:>6}      {:>6}\n",
        "", "WINDOW", "", "CUMUL"
    ));
    let mut by_weight: Vec<_> = stats.totals.iter().collect();
    by_weight.sort_by_key(|(_, weight)| std::cmp::Reverse(**weight));
    let mut next_previous = HashMap::new();
    for (function, total) in by_weight {
        let pct = 100.0 * *total as f64 / stats.grand as f64;
        let cumulative_pct = cumulative_stats
            .totals
            .get(function)
            .map(|w| 100.0 * *w as f64 / cumulative_stats.grand as f64)
            .unwrap_or(0.0);
        let trend = match previous.get(function) {
            Some(prior) if pct - prior > 2.0 => "▲",
            Some(prior) if prior - pct > 2.0 => "▼",
            Some(_) => "─",
            None => " ",
        };
        next_previous.insert(function.clone(), pct);
        out.push_str(&format!(
            "{function:<26} {} {pct:5.1}% {trend}    {cumulative_pct:5.1}%\n",
            bar(pct, 22)
        ));
        let mut cause_rows: Vec<_> = stats
            .causes
            .get(function)
            .map(|map| map.iter().collect())
            .unwrap_or_default();
        cause_rows.sort_by_key(|(_, weight)| std::cmp::Reverse(**weight));
        for (cause, weight) in cause_rows.into_iter().take(4) {
            let cause_pct = 100.0 * *weight as f64 / stats.grand as f64;
            out.push_str(&format!(
                "    {cause:<25} {} {cause_pct:5.1}%\n",
                bar(cause_pct, 22)
            ));
        }
    }
    *previous = next_previous;
    out
}
