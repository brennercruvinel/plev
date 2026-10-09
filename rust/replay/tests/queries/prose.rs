//! the text answers: the causal chain says, one labelled line each, what
//! the thesis asks (which control, its state, who initiated it, which
//! action, what was observed, the precondition, what came before, the
//! verdict, what changed, the result), in plain english.

use replay::{Inspector, prose};

use crate::thesis::record;

type Fallible = Result<(), Box<dyn std::error::Error>>;

#[test]
fn the_stale_submit_answers_every_question_of_the_thesis() -> Fallible {
    let thesis = record();
    let inspector = Inspector::new(&thesis.session);
    let why = inspector.why(0, thesis.stale)?.ok_or("the stale submit")?;
    let answer = text::why(&why);
    for (label, says) in [
        ("control", r#"commit.submit: button "Commit", enabled"#),
        ("initiated by", "agent:reviewer"),
        ("action", "submit, no parameter"),
        ("observed", "+0.000s, the whole app, at revision"),
        (
            "precondition",
            "what the action reads, as agent:reviewer saw it:",
        ),
        (
            "before",
            "from the observation (+0.000s) to the request (+0.300s):",
        ),
        ("verdict", "rejected at +0.300s: stale"),
        ("what changed", "changes:staged: removed B"),
        ("acted", "human, request 1: unstage on changes"),
        ("result", "nothing ran: the refusal came before any effect"),
    ] {
        let line = answer
            .lines()
            .find(|l| l.trim_start().starts_with(label))
            .ok_or_else(|| format!("no {label:?} line in\n{answer}"))?;
        assert!(line.contains(says), "{label}: {line:?} lacks {says:?}");
    }
    for says in [
        "changes:staged = {A, B, C}",
        "+0.100s changes:staged: removed B",
        r#"by human, request 1: unstage on changes with "B" (executing at +0.100s)"#,
        "commit.message:text did not move",
        "+0.100s input: pointer_down on changes",
        "a correlation in time, not a declared write",
    ] {
        assert!(answer.contains(says), "{says:?} in\n{answer}");
    }
    Ok(())
}

#[test]
fn the_completed_submit_says_its_sha_and_files() -> Fallible {
    let thesis = record();
    let why = Inspector::new(&thesis.session)
        .why(0, thesis.done)?
        .ok_or("the completed submit")?;
    let answer = text::why(&why);
    assert!(
        answer.contains("verdict       accepted at +0.500s"),
        "{answer}"
    );
    assert!(
        answer.contains("result        completed at +0.500s: commit = \""),
        "{answer}"
    );
    assert!(answer.contains("files = [A, C]"), "{answer}");
    assert!(
        !answer.contains("note"),
        "no move, no note on the correlation"
    );
    Ok(())
}

#[test]
fn a_persons_request_says_why_no_observation_is_recorded() -> Fallible {
    let thesis = record();
    let why = Inspector::new(&thesis.session)
        .why(0, thesis.unstage)?
        .ok_or("the unstage")?;
    let answer = text::why(&why);
    for says in [
        "not recorded: no observation by human before the request",
        "a person acts on the frame on screen",
        "the recorded state was at revision",
        "from the last state at that revision (+0.000s)",
    ] {
        assert!(answer.contains(says), "{says:?} in\n{answer}");
    }
    Ok(())
}

#[test]
fn the_summary_lists_every_request_and_the_refusal_with_its_why() -> Fallible {
    let thesis = record();
    let summary = Inspector::new(&thesis.session).summary()?;
    assert_eq!(summary.requests.len(), 3);
    assert_eq!(summary.refusals.len(), 1);
    assert_eq!(
        (summary.start, summary.end, summary.duration),
        (1_000, 1_500, 500)
    );
    let answer = text::summary(&summary);
    for says in [
        "span      +0.000s to +0.500s",
        "requests  3",
        "rejected: stale",
        "refusals  1",
        "request 2, submit on commit.submit by agent:reviewer, rejected at +0.300s: stale",
        "changes:staged: removed B",
        "changes:staged moved at +0.100s by human, request 1",
    ] {
        assert!(answer.contains(says), "{says:?} in\n{answer}");
    }
    Ok(())
}

#[test]
fn the_timeline_and_the_control_read_as_lines() -> Fallible {
    let thesis = record();
    let inspector = Inspector::new(&thesis.session);
    let timeline = text::timeline(&inspector.timeline());
    assert_eq!(
        timeline.lines().filter(|l| l.starts_with('+')).count(),
        thesis.session.events().len()
    );
    assert!(timeline.contains("observed  agent:reviewer observed the whole app"));
    assert!(
        timeline.contains(
            "stage     request 2 rejected: stale (agent:reviewer, submit on commit.submit)"
        )
    );
    let control = text::control(&inspector.control(&crate::apps::id("commit.submit"), 1_300)?);
    for says in [
        "control commit.submit at +0.300s",
        r#"node          commit.submit: button "Commit", enabled"#,
        "actions       submit (reads changes:staged, commit.message:text)",
        "requests      1 touched it up to +0.300s",
        "  request 2 (origin 0): submit on commit.submit by agent:reviewer",
    ] {
        assert!(control.contains(says), "{says:?} in\n{control}");
    }
    assert!(control.contains(&format!(", in build {}", thesis.session.build().git)));
    Ok(())
}

/// a move no receipt explains says it came from outside the app; two
/// receipts in one window say the answer is ambiguous and name both.
#[test]
fn outside_and_ambiguous_moves_say_so() -> Fallible {
    let mut lab = crate::apps::Lab::new(&["A", "B", "C", "D"], "fix login", 0);
    let seen = lab.observe(10, crate::apps::agent());
    lab.outside_unstages(20, "D");
    for (t, actor, file) in [
        (30, accord::Actor::Human, "B"),
        (31, accord::Actor::Assistive, "C"),
    ] {
        let req =
            accord::Request::observe(&lab.look(), actor, "changes", "unstage", Some(file.into()))?;
        lab.dispatcher.dispatch(&mut lab.git, req, t);
    }
    lab.rec.snapshot(31, &lab.look());
    let submit = lab.act(
        40,
        &seen,
        crate::apps::agent(),
        "commit.submit",
        "submit",
        None,
    );
    let session = lab.session();
    let why = Inspector::new(&session)
        .why(0, submit)?
        .ok_or("the submit")?;
    let answer = text::why(&why);
    for says in [
        "+0.020s changes:staged: removed D",
        "by no request: no receipt ran between the state before and this change, so it came \
         from outside the app",
        "+0.031s changes:staged: removed B, C",
        "by one of 2 receipts that ran then (ambiguous): human, request 1: unstage on changes \
         with \"B\" (executing at +0.030s); assistive, request 2",
        // both were checked on the state the outside change left, not on the
        // one the agent saw: the dispatcher's acted names neither, and the
        // recording still shows them.
        "acted         the refusal names nobody",
    ] {
        assert!(answer.contains(says), "{says:?} in\n{answer}");
    }
    Ok(())
}
