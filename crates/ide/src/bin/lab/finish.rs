//! the end of a plev or scripted run: the session reopened from its file
//! and checked against the real git, the story of the agent told against
//! the person, what the agent asked her and what she answered, the costs,
//! and the report written as json and printed.

use rec::Session;
use sem::{SemId, Stage, Value};
use serde_json::{Value as Json, json};

use crate::asked;
use crate::host::{Intervention, Trigger};
use crate::lab::Lab;
use crate::refused;
use crate::repo::{TASK, TASK_FILES, TASK_MESSAGE};
use crate::report::{self, Truth, human, ms, per_call_us, row};
use crate::story;

impl Lab {
    pub fn finish(self, ended_by: &'static str) -> Result<i32, String> {
        let Lab {
            args,
            mut app,
            rec,
            log,
            port,
            person,
            served,
            acts,
            costs,
            projection_error,
            timing,
            started,
            head_before,
            first_message,
            submit_completed,
            stale,
            intervention_recorded_at,
            frame_error,
            scripted,
        } = self;
        let end = app.bench.now();
        let duration = started.elapsed();
        let traffic = port.traffic();
        // the port closes first: the scripted agent, if still reading,
        // sees the end and returns.
        let port_path = port.path().to_owned();
        drop(port);
        let script = scripted.map(|h| match h.join() {
            Ok(Ok(s)) => (s.steps, s.error, s.refused),
            Ok(Err(e)) => (Vec::new(), Some(e), Vec::new()),
            Err(_) => (
                Vec::new(),
                Some("the scripted agent panicked".to_owned()),
                Vec::new(),
            ),
        });

        let file = args.out.join("session.monster");
        let persisted = rec
            .persist(&file)
            .map_err(|e| format!("{}: {e}", file.display()))?;
        let session =
            Session::open(&file).map_err(|e| format!("reopen {}: {e}", file.display()))?;
        let submit = SemId::parse("commit.submit").map_err(|e| e.to_string())?;
        let completed = session
            .receipts_for(&submit, end.saturating_add(1))
            .into_iter()
            .find_map(|r| match r.receipt.stage() {
                Some(Stage::Completed(outcome)) => {
                    let sha = match outcome.result.get("commit") {
                        Some(Value::Text(sha)) => sha.clone(),
                        _ => String::new(),
                    };
                    let files = match outcome.result.get("files") {
                        Some(Value::List(files)) => files.clone(),
                        _ => Vec::new(),
                    };
                    Some((sha, files))
                }
                _ => None,
            });
        let truth = Truth::read(&app.repo, &head_before, Some(completed))?;

        let receipts = log.receipts();
        let refusals = refused::Account {
            met: script.as_ref().map(|(_, _, met)| met.as_slice()),
            taken: traffic.acts,
            dispatched: acts.len(),
        };
        let acts = story::acts(&receipts, &acts, &served);
        let questions = asked::questions(&receipts, &person.answers.waited, &acts);
        let over = asked::over_the_person(&receipts);
        let no_commit_because = (!truth.committed()).then(|| {
            asked::no_commit_because(&questions)
                .unwrap_or_else(|| "no submit of an agent completed".to_owned())
        });
        let answered = |allowed: bool| {
            questions
                .iter()
                .filter(|q| q.answer == Some(if allowed { "allowed" } else { "declined" }))
                .count()
        };
        let intervention = person.fired.as_ref();
        let measured = story::stale_measurement(&acts, intervention);
        let recovered = story::recovered(&acts);
        let stats = rec.stats();
        let t = timing;
        let run_ms = ms(duration);
        let hello_to_commit = match (first_message, submit_completed) {
            (Some(a), Some(b)) => Some(b.saturating_sub(a)),
            _ => None,
        };
        let mut notes: Vec<String> = Vec::new();
        if let Some(e) = projection_error.borrow().as_ref() {
            notes.push(format!("a projection did not validate: {e}"));
        }
        if let Some(e) = &frame_error {
            notes.push(format!("a frame was not recorded: {e}"));
        }
        if app.gpu.is_none() {
            notes.push("no GPU adapter: frames were built and recorded, not drawn".to_owned());
        }
        if let Some(i) = intervention
            && !i.revision_matched
        {
            notes.push(match i.trigger {
                Trigger::Observation => {
                    "the live projection at the intervention was not the revision served".into()
                }
                Trigger::Submit => {
                    "the submit that brought the person named a revision the app had left".into()
                }
            });
        }
        if let Some((_, Some(e), _)) = &script {
            notes.push(format!("the scripted agent: {e}"));
        }
        if let Some(why) = &no_commit_because {
            notes.push(format!("no commit: {why}"));
        }
        for q in questions.iter().filter(|q| q.missed.is_some()) {
            notes.push(format!(
                "the person did not answer request {}: {}",
                q.request,
                q.missed.as_deref().unwrap_or_default()
            ));
        }
        notes.push(
            "stale measurement: by value. valid when a submit that reached the dispatcher after \
             the person's act carried the changes:staged of before it (seen) while the app held \
             another (live), whatever revision it named"
                .to_owned(),
        );
        notes.push(
            "form: what every act rested on, as the port counted the acts (acts_form). \
             revision_served_index, observations_between, fresh_before: the revision an act \
             named, against the observations served, as information. a revision is a hash of \
             what the app projects, so a world put back as it was has the revision it had then, \
             and a revision a terminal stage carried is served by no observation"
                .to_owned(),
        );
        notes.push(
            "questions: every request of an agent that waited for a person, question null when \
             the action's policy asked her; answer_ms from awaiting_confirmation to the stage \
             that followed; then: the agent's next act after the answer. over_the_person: the \
             acts of an agent, with no question, that ran on a control a person acted on, \
             reading a state her act read, after a stale refusal of that agent named her act"
                .to_owned(),
        );
        let contract = json!({
            "drain_total_ms": ms(t.drain),
            "drains": t.drains,
            "drain_busy_ms": ms(t.drain_busy),
            "drains_busy": t.drains_busy,
            "per_message_us": per_call_us(t.drain_busy, t.messages),
            "fraction_of_run": t.drain.as_secs_f64() / duration.as_secs_f64().max(1e-9),
            "project_calls": costs.project_calls.get(),
            "project_per_call_us": per_call_us(costs.project.get(), costs.project_calls.get()),
            "dispatch_calls": costs.dispatch_calls.get(),
            "dispatch_per_call_us": per_call_us(costs.dispatch.get(), costs.dispatch_calls.get()),
            "lab_look_calls": costs.look_calls.get(),
            "lab_look_per_call_us": per_call_us(costs.look.get(), costs.look_calls.get()),
            "loop_snapshot_calls": t.snapshots,
            "loop_snapshot_per_call_us": per_call_us(t.snapshot, t.snapshots),
        });
        let recording = json!({
            "snapshot_calls": t.rec_snapshots,
            "snapshot_per_call_us": per_call_us(t.rec_snapshot, t.rec_snapshots),
            "frame_calls": t.rec_frames,
            "frame_per_call_us": per_call_us(t.rec_frame, t.rec_frames),
            "build_calls": t.builds,
            "build_per_call_us": per_call_us(t.build, t.builds),
            "draw_calls": t.draws,
            "draw_per_call_us": per_call_us(t.draw, t.draws),
            "gpu": app.gpu.is_some(),
            "total_ms": ms(t.rec_snapshot + t.rec_frame),
            "fraction_of_run": (t.rec_snapshot + t.rec_frame).as_secs_f64() / duration.as_secs_f64().max(1e-9),
            "stats": report::stats_json(&stats),
        });
        let visual_bytes = persisted.index_bytes + persisted.chunks_bytes + persisted.assets_bytes;
        let report = json!({
            "arm": args.arm.name(),
            "human": args.human.describe(),
            "human_path": args.human.path(),
            "app": args.app,
            "agent_port": port_path.display().to_string(),
            "task": TASK,
            "task_files": TASK_FILES,
            "task_message": TASK_MESSAGE,
            "run": {
                "duration_ms": run_ms,
                "ended_by": ended_by,
                "first_served_message_ms": first_message,
                "submit_completed_ms": submit_completed,
                "first_message_to_commit_ms": hello_to_commit,
                "messages_served": timing.messages,
                "timeout_s": args.timeout.as_secs_f64(),
                "grace_s": args.grace.as_secs_f64(),
                "no_commit_because": no_commit_because,
            },
            "person_answers": {
                "scenario": args.answer.name(),
                "after_ms": u64::try_from(args.answer_after.as_millis()).unwrap_or(u64::MAX),
                "allowed": answered(true),
                "declined": answered(false),
                "unanswered": questions.iter().filter(|q| q.answer.is_none()).count(),
            },
            "traffic": report::traffic_json(&traffic),
            "agent": {
                "observations": story::served_json(&served),
                "acts": story::acts_json(&acts, story::form(&traffic)),
                "acts_form": story::forms_text(&traffic),
                "acts_by_form": {
                    "from_revision": traffic.acts_from_revision,
                    "observed": traffic.acts_observed,
                },
                "round_trips": traffic.msgs_in,
                "stale_rejections": stale.len(),
                "recovered": recovered,
                "stale_measurement_valid": measured.valid,
                "stale_measurement": measured.text,
                "stale_measured": measured.json(),
                "script": script.as_ref().map(|(steps, _, _)| steps.clone()),
                "protocol_refusals": refusals.json(),
                "questions_asked": questions.iter().filter(|q| q.text.is_some()).count(),
                "questions": asked::json(&questions),
                "over_the_person": asked::over_json(&over),
            },
            "intervention": intervention.map(intervention_json),
            "intervention_recorded_at": intervention_recorded_at,
            "stale": stale.iter().map(|s| json!({
                "t": s.t,
                "request": s.request.to_string(),
                "head": s.head,
                "changes": s.changes.iter().map(|c| serde_json::to_value(c).unwrap_or(Json::Null)).collect::<Vec<_>>(),
                "acted": story::acted_json(&s.acted),
            })).collect::<Vec<_>>(),
            "cost": { "contract": contract, "recording": recording },
            "session": {
                "file": file.display().to_string(),
                "bytes": persisted.bytes,
                "events_bytes": persisted.events_bytes,
                "index_bytes": persisted.index_bytes,
                "chunks_bytes": persisted.chunks_bytes,
                "assets_bytes": persisted.assets_bytes,
                "visual_bytes": visual_bytes,
                "events": persisted.events,
                "dropped": persisted.dropped,
                "chunks": persisted.chunks,
                "chunk_sizes": session.chunk_sizes(),
                "frames": session.frame_times().len(),
                "span": session.span(),
                "reopened": true,
            },
            "truth": truth.json(),
            "notes": notes,
            "exit": truth.exit_code(),
        });
        report::write(&args.out, &report)?;
        // the app outlives the print: the repository must not be removed
        // before git was read.
        let _ = &mut app;

        println!();
        report::print_table(
            "run",
            &[
                row("ended by", ended_by),
                row("duration", &format!("{run_ms:.0} ms")),
                row(
                    "first served message to commit",
                    &hello_to_commit.map_or("none".to_owned(), |ms| format!("{ms} ms")),
                ),
                row(
                    "no commit because",
                    no_commit_because.as_deref().unwrap_or("(a commit exists)"),
                ),
            ],
        );
        report::print_table("traffic", &report::traffic_rows(&traffic));
        let mut agent = vec![
            row("observations served", &served.len().to_string()),
            row("acts", &acts.len().to_string()),
            row("acts rested on", &story::forms_text(&traffic)),
            row("round trips (messages in)", &traffic.msgs_in.to_string()),
            row("stale rejections", &stale.len().to_string()),
            row("recovered after stale", &recovered.to_string()),
            row("stale_measurement_valid", &measured.valid.to_string()),
            row("stale measurement", &measured.text),
        ];
        for a in &acts {
            let values = match (&a.seen_staged, a.action == "submit") {
                (Some(seen), true) => format!(
                    ", staged seen {seen:?}, live {}",
                    a.live_staged
                        .as_ref()
                        .map_or("unknown".to_owned(), |l| format!("{l:?}"))
                ),
                _ => String::new(),
            };
            agent.push(row(
                &format!("act {} {}", a.target, a.action),
                &format!(
                    "{} at {} ms, request {}, revision {} ({}, {} observation(s) between, \
                     fresh_before {}){values}",
                    a.terminal,
                    a.t,
                    a.request,
                    a.revision,
                    story::form(&traffic).unwrap_or("form mixed"),
                    a.observations_between,
                    a.fresh_before
                ),
            ));
        }
        report::print_table("agent", &agent);
        report::print_table("refusals of the protocol", &refusals.rows());
        report::print_table("the person's answers", &person.answers.rows());
        report::print_table("what the agent asked", &asked::rows(&questions, &over));
        match intervention {
            Some(i) => report::print_table("intervention", &intervention_rows(i)),
            None => report::print_table("intervention", &[row("fired", "false")]),
        }
        for s in &stale {
            report::print_table(
                "stale rejection",
                &[
                    row("t", &format!("{} ms", s.t)),
                    row("request", &s.request.to_string()),
                    row("head at that instant", &s.head),
                    row("changes", &format!("{:?}", s.changes)),
                    row("acted on what it rested on", &story::acted_text(&s.acted)),
                ],
            );
        }
        let per = |total, calls| {
            format!(
                "{:.1} us per call, {} calls",
                per_call_us(total, calls),
                calls
            )
        };
        report::print_table(
            "cost of the contract",
            &[
                row(
                    "time inside drain",
                    &format!("{} over {} drains", human(t.drain), t.drains),
                ),
                row(
                    "per message served",
                    &format!(
                        "{:.1} us ({} messages over {} busy drains)",
                        per_call_us(t.drain_busy, t.messages),
                        t.messages,
                        t.drains_busy
                    ),
                ),
                row(
                    "fraction of the run",
                    &format!(
                        "{:.4}",
                        t.drain.as_secs_f64() / duration.as_secs_f64().max(1e-9)
                    ),
                ),
                row(
                    "bench.snapshot() for the port",
                    &per(costs.project.get(), costs.project_calls.get()),
                ),
                row(
                    "bench.dispatch() for the port",
                    &per(costs.dispatch.get(), costs.dispatch_calls.get()),
                ),
                row(
                    "the lab's look at each submit (out of drain)",
                    &per(costs.look.get(), costs.look_calls.get()),
                ),
            ],
        );
        report::print_table(
            "cost of the recording",
            &[
                row("rec.snapshot", &per(t.rec_snapshot, t.rec_snapshots)),
                row("rec.frame", &per(t.rec_frame, t.rec_frames)),
                row("scene build", &per(t.build, t.builds)),
                row(
                    "gpu draw",
                    &if app.gpu.is_some() {
                        per(t.draw, t.draws)
                    } else {
                        "no GPU adapter".to_owned()
                    },
                ),
                row(
                    "events / frames / unchanged frames",
                    &format!(
                        "{} / {} / {}",
                        stats.events, stats.frames, stats.frames_unchanged
                    ),
                ),
                row(
                    "snapshots full / changed / unchanged",
                    &format!(
                        "{} / {} / {}",
                        stats.snapshots_full, stats.snapshots_changed, stats.snapshots_unchanged
                    ),
                ),
                row(
                    "ring bytes semantic / visual",
                    &format!("{} / {}", stats.semantic_bytes, stats.visual_bytes),
                ),
            ],
        );
        report::print_table(
            "session",
            &[
                row("file", &file.display().to_string()),
                row("bytes", &persisted.bytes.to_string()),
                row(
                    "semantic track (events)",
                    &format!(
                        "{} bytes, {} events",
                        persisted.events_bytes, persisted.events
                    ),
                ),
                row(
                    "visual track (index + chunks + assets)",
                    &format!(
                        "{} bytes, {} chunks, {} frames",
                        visual_bytes,
                        persisted.chunks,
                        session.frame_times().len()
                    ),
                ),
                row(
                    "chunk sizes (bytes, assets)",
                    &format!("{:?}", session.chunk_sizes()),
                ),
                row("reopened", "true"),
            ],
        );
        let mut truth_rows = truth.rows();
        truth_rows.push(row("exit", &truth.exit_code().to_string()));
        report::print_table("truth", &truth_rows);
        println!("git log --oneline");
        for line in truth.log.lines() {
            println!("  {line}");
        }
        println!("git status --porcelain");
        for line in truth.status.lines() {
            println!("  {line}");
        }
        println!("notes");
        for note in &notes {
            println!("  {note}");
        }
        println!("report: {}", args.out.join("report.json").display());
        Ok(truth.exit_code())
    }
}

fn intervention_json(i: &Intervention) -> Json {
    json!({
        "t": i.t,
        "trigger": i.trigger.name(),
        "served_index": i.served_index,
        "path": i.path,
        "revision_served": i.revision_served.to_string(),
        "head": i.head,
        "request": i.request.to_string(),
        "revision_matched": i.revision_matched,
        "staged_before": i.staged_before,
        "staged_after": i.staged_after,
    })
}

fn intervention_rows(i: &Intervention) -> Vec<(String, String)> {
    vec![
        row("fired", "true"),
        row("t", &format!("{} ms", i.t)),
        row("trigger", i.trigger.name()),
        row("path", &i.path),
        row(
            match i.trigger {
                Trigger::Observation => "revision served just before",
                Trigger::Submit => "revision the submit named",
            },
            &i.revision_served.to_string(),
        ),
        row("head at that instant", &i.head),
        row("person's request", &i.request.to_string()),
        row("staged before", &format!("{:?}", i.staged_before)),
        row("staged after", &format!("{:?}", i.staged_after)),
    ]
}
