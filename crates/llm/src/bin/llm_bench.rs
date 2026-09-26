//! llm-bench: one implementation per run, no window. loads the model, runs
//! the same prompt `--rounds` times through the streaming interface, each
//! round exactly `--new` tokens (the eos does not stop it, so every round
//! and every implementation decodes the same count), and prints the load
//! time, the time to the first token, the decode rate, the peak memory, the
//! binary size, the dtype and device, and the first tokens.
//!
//! usage: llm-bench --impl candle|burn [--model DIR] [--device metal|cpu]
//!        [--rounds 5] [--new 128] [--show 32] [--prompt TEXT]
//!        [--layout row|col] (burn only; the default depends on the device)
//! the model directory defaults to PLEV_LLM_MODEL.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use llm::stream::{self, Request};
use llm::text::Text;
use llm::{Device, Event, Model, chat};

const PROMPT: &str =
    "Write a short story about a lighthouse keeper who finds a message in a bottle.";

struct Args {
    imp: String,
    model: PathBuf,
    device: Device,
    rounds: usize,
    new: usize,
    show: usize,
    prompt: String,
    layout: Option<String>,
}

fn value(it: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    it.next().with_context(|| format!("{flag} needs a value"))
}

fn parse() -> Result<Args> {
    let mut args = Args {
        imp: String::new(),
        model: std::env::var_os(llm::MODEL_ENV)
            .map(PathBuf::from)
            .unwrap_or_default(),
        device: Device::Metal,
        rounds: 5,
        new: 128,
        show: 32,
        prompt: PROMPT.to_string(),
        layout: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--impl" => args.imp = value(&mut it, &flag)?,
            "--model" => args.model = value(&mut it, &flag)?.into(),
            "--device" => {
                let name = value(&mut it, &flag)?;
                args.device = Device::parse(&name).with_context(|| format!("device {name}"))?;
            }
            "--rounds" => args.rounds = value(&mut it, &flag)?.parse()?,
            "--new" => args.new = value(&mut it, &flag)?.parse()?,
            "--show" => args.show = value(&mut it, &flag)?.parse()?,
            "--prompt" => args.prompt = value(&mut it, &flag)?,
            "--layout" => args.layout = Some(value(&mut it, &flag)?),
            other => bail!("unknown argument {other}"),
        }
    }
    if args.model.as_os_str().is_empty() {
        bail!("no model: pass --model DIR or set {}", llm::MODEL_ENV);
    }
    if args.imp.is_empty() {
        let built: Vec<&str> = [
            (cfg!(feature = "candle"), "candle"),
            (cfg!(feature = "burn"), "burn"),
        ]
        .into_iter()
        .filter_map(|(on, name)| on.then_some(name))
        .collect();
        match built.as_slice() {
            [one] => args.imp = one.to_string(),
            _ => bail!("pass --impl candle|burn (built: {built:?})"),
        }
    }
    if args.rounds == 0 || args.new < 2 {
        bail!("at least one round of two tokens");
    }
    Ok(args)
}

fn load(args: &Args) -> Result<Box<dyn Model>> {
    let (dir, device) = (args.model.as_path(), args.device);
    if args.layout.is_some() && args.imp != "burn" {
        bail!("--layout is for burn only");
    }
    match args.imp.as_str() {
        #[cfg(feature = "candle")]
        "candle" => Ok(Box::new(llm::candle::CandleModel::load(dir, device)?)),
        #[cfg(feature = "burn")]
        "burn" => {
            use llm::burn::LinearLayout;
            let layout = match args.layout.as_deref() {
                None => llm::burn::layout_for(device),
                Some("row") => LinearLayout::Row,
                Some("col") => LinearLayout::Col,
                Some(other) => bail!("layout {other}: row or col"),
            };
            Ok(llm::burn::load_with(dir, device, layout)?)
        }
        other => bail!("{other} is not built in; build with --features {other}"),
    }
}

struct Round {
    ttft: Duration,
    rate: f64,
    ids: Vec<u32>,
    texts: Vec<String>,
}

fn round(
    model: &Arc<Mutex<Box<dyn Model>>>,
    text: &Text,
    prompt: &[u32],
    new: usize,
) -> Result<Round> {
    let req = Request {
        prompt: prompt.to_vec(),
        max_new: new,
        eos: Vec::new(),
    };
    let start = Instant::now();
    let mut stream = stream::spawn(model.clone(), req, Box::new(text.detok()));
    let (mut first, mut last) = (None, start);
    let (mut ids, mut texts) = (Vec::new(), Vec::new());
    while let Some(event) = stream.blocking_next() {
        match event {
            Event::Token(t) => {
                let now = Instant::now();
                first.get_or_insert(now);
                last = now;
                ids.push(t.id);
                texts.push(t.text);
            }
            Event::Done(_) => break,
            Event::Failed(err) => return Err(err.into()),
        }
    }
    let first = first.context("the round produced no token")?;
    if ids.len() != new {
        bail!("the round produced {} tokens, not {new}", ids.len());
    }
    let decode = (last - first).as_secs_f64();
    Ok(Round {
        ttft: first - start,
        rate: (ids.len() - 1) as f64 / decode,
        ids,
        texts,
    })
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        values[mid]
    } else {
        (values[mid - 1] + values[mid]) / 2.0
    }
}

/// Peak resident set and peak physical footprint, in bytes. The footprint is
/// what Activity Monitor shows; on Apple silicon it also counts the Metal
/// buffers, which the resident set may not.
fn peak_memory() -> (u64, u64) {
    // SAFETY: an all-zero rusage is a valid value of this plain C struct.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    // SAFETY: getrusage writes one rusage into the value we own and pass by
    // a unique pointer.
    let rss = if unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) } == 0 {
        let max = usage.ru_maxrss.max(0) as u64;
        // macOS reports bytes, linux kilobytes.
        if cfg!(target_os = "macos") {
            max
        } else {
            max * 1024
        }
    } else {
        0
    };
    (rss, footprint())
}

#[cfg(target_os = "macos")]
fn footprint() -> u64 {
    // SAFETY: an all-zero rusage_info_v4 is a valid value of this plain C struct.
    let mut info: libc::rusage_info_v4 = unsafe { std::mem::zeroed() };
    let buffer = (&mut info as *mut libc::rusage_info_v4).cast::<libc::rusage_info_t>();
    // SAFETY: with RUSAGE_INFO_V4 the kernel writes one rusage_info_v4 into
    // the buffer, which points at a value of exactly that type that we own.
    let rc = unsafe { libc::proc_pid_rusage(libc::getpid(), libc::RUSAGE_INFO_V4, buffer) };
    if rc == 0 {
        info.ri_lifetime_max_phys_footprint
    } else {
        0
    }
}

#[cfg(not(target_os = "macos"))]
fn footprint() -> u64 {
    0
}

fn mb(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_000_000.0)
}

fn main() -> Result<()> {
    let args = parse()?;
    let binary = std::env::current_exe()?.metadata()?.len();

    let started = Instant::now();
    let text = Text::from_dir(&args.model)?;
    let tokenizer = started.elapsed();
    let prompt = text.encode(&chat::ask(&args.prompt))?;

    let started = Instant::now();
    let model = load(&args)?;
    let loaded = started.elapsed();
    let info = model.info();
    let model = Arc::new(Mutex::new(model));

    let mut rounds = Vec::with_capacity(args.rounds);
    for _ in 0..args.rounds {
        rounds.push(round(&model, &text, &prompt, args.new)?);
    }
    let (rss, foot) = peak_memory();

    println!("impl: {}", info.backend);
    println!("device: {}", info.device);
    println!("dtype: {}", info.dtype);
    println!("binary: {} ({binary} bytes)", mb(binary));
    println!("tokenizer load: {:.3} s", tokenizer.as_secs_f64());
    println!("model load: {:.3} s", loaded.as_secs_f64());
    println!("prompt: {} tokens", prompt.len());
    println!("rounds: {} x {} new tokens", args.rounds, args.new);
    for (i, r) in rounds.iter().enumerate() {
        println!(
            "round {}: ttft {:.1} ms, decode {:.2} tok/s",
            i + 1,
            r.ttft.as_secs_f64() * 1000.0,
            r.rate
        );
    }
    let ttft = median(
        rounds
            .iter()
            .map(|r| r.ttft.as_secs_f64() * 1000.0)
            .collect(),
    );
    let rate = median(rounds.iter().map(|r| r.rate).collect());
    println!("median: ttft {ttft:.1} ms, decode {rate:.2} tok/s");
    println!("peak rss: {} ({rss} bytes)", mb(rss));
    println!("peak footprint: {} ({foot} bytes)", mb(foot));
    let same = rounds.iter().all(|r| r.ids == rounds[0].ids);
    println!(
        "same tokens in every round: {}",
        if same { "yes" } else { "no" }
    );
    let show = args.show.min(args.new);
    println!("first {show} ids: {:?}", &rounds[0].ids[..show]);
    println!("first {show} text: {:?}", rounds[0].texts[..show].concat());
    Ok(())
}
