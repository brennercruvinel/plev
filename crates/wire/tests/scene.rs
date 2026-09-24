//! the scene channel (task-79): the forms a watcher reads, pinned; a
//! watch served by the server in process (the first frame whole, then
//! only what moved, an asset once while frames reference it, a blob in
//! parts under the line limit, a body too large for its line as a blob);
//! the reader putting lines back together and refusing them out of
//! order; and over a real socket, beside an observation, with a watcher
//! that stops reading cut off while the app never waits.

#![cfg(unix)]

use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use sem::{
    Actor, ReceiptSink, Request, RequestId, Revision, Role, SemId, Snapshot, SnapshotBuilder,
};
use wire::{
    AgentPort, Ask, BlobReply, Bound, Client, ClientMsg, Config, Host, LayerBody, MAX_BLOB,
    MAX_LINE, Observe, Offer, OfferLayer, Outgoing, ProtoError, Reply, SCENE_PART, SCENE_QUEUE,
    SceneAsk, SceneLayer, SceneReader, SceneReply, Server, ServerMsg, decode_client, decode_server,
    encode_client, encode_server,
};

/// an app that projects one node and takes no request; it keeps what it
/// was told was observed, as a recording would.
#[derive(Default)]
struct Still {
    observed: Vec<(Actor, Option<SemId>, Revision)>,
}

impl Host for Still {
    fn project(&self) -> Option<Snapshot> {
        let mut b = SnapshotBuilder::new();
        b.node("todo", Role::Region, "todo");
        b.finish().ok()
    }

    fn dispatch(&mut self, _req: Request) -> Option<RequestId> {
        None
    }

    fn add_sink(&mut self, _sink: Box<dyn ReceiptSink>) {}

    fn observed(&mut self, actor: &Actor, scope: Option<&SemId>, revision: Revision) {
        self.observed
            .push((actor.clone(), scope.cloned(), revision));
    }
}

fn hello(server: &mut Server, conn: u64, name: &str) {
    let out = server.handle(
        conn,
        ClientMsg {
            id: format!("h{conn}"),
            ask: Ask::Hello { name: name.into() },
        },
        &mut Still::default(),
    );
    assert!(matches!(out[0].msg.reply, Reply::Welcome { .. }));
}

/// a server whose scenes the app's person shares.
fn shared() -> Server {
    let mut s = Server::new("toy", 8);
    s.share_scenes(true);
    s
}

fn bytes(n: usize, seed: u8) -> Arc<[u8]> {
    (0..n)
        .map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))
        .collect::<Vec<u8>>()
        .into()
}

fn layer(key: u32, hash: u64, len: usize) -> OfferLayer {
    OfferLayer {
        key,
        hash,
        body: bytes(len, hash as u8),
        patch: None,
    }
}

fn bound(id: &str, x: f32) -> Bound {
    Bound {
        id: SemId::parse(id).unwrap(),
        rect: [x, 2.0, 30.0, 40.0],
    }
}

/// a frame of three layers, one asset, one bound control.
fn offer() -> Offer {
    Offer {
        head: bytes(28, 7),
        layers: vec![layer(2, 20, 40), layer(0, 10, 300), layer(1, 30, 80)],
        assets: vec![(0xa5, bytes(600, 1))],
        bounds: vec![bound("todo.input", 1.0)],
    }
}

fn watch(server: &mut Server, conn: u64, window: &str) {
    hello(server, conn, &format!("agent{conn}"));
    let out = server.handle(
        conn,
        ClientMsg {
            id: format!("w{conn}"),
            ask: Ask::Scene(SceneAsk {
                window: window.to_owned(),
            }),
        },
        &mut Still::default(),
    );
    assert!(out.is_empty(), "a watch answers with frames only: {out:?}");
}

fn frames(out: &[Outgoing]) -> Vec<&SceneReply> {
    out.iter()
        .filter_map(|o| match &o.msg.reply {
            Reply::Scene(f) => Some(f),
            _ => None,
        })
        .collect()
}

fn blobs(out: &[Outgoing]) -> Vec<&BlobReply> {
    out.iter()
        .filter_map(|o| match &o.msg.reply {
            Reply::SceneBlob(b) => Some(b),
            _ => None,
        })
        .collect()
}

/// the lines as the port writes them, each checked under the limit.
fn lines(out: &[Outgoing]) -> Vec<String> {
    out.iter()
        .map(|o| {
            let line = encode_server(&o.msg);
            assert!(line.len() < MAX_LINE, "a line of {} bytes", line.len());
            line
        })
        .collect()
}

#[test]
fn a_scene_request_is_pinned_and_watches_the_main_window_by_default() {
    let asks = [
        (r#"{"v":1,"id":"9","type":"scene","window":"page"}"#, "page"),
        (r#"{"v":1,"id":"9","type":"scene"}"#, "main"),
    ];
    for (line, window) in asks {
        let msg = decode_client(line).unwrap();
        assert_eq!(
            msg.ask,
            Ask::Scene(SceneAsk {
                window: window.into()
            })
        );
    }
    let line = encode_client(&ClientMsg {
        id: "9".into(),
        ask: Ask::Scene(SceneAsk {
            window: "main".into(),
        }),
    });
    assert_eq!(line, r#"{"v":1,"id":"9","type":"scene","window":"main"}"#);
}

#[test]
fn the_lines_of_a_watch_are_pinned() {
    let frame = ServerMsg {
        id: Some("9".into()),
        reply: Reply::Scene(SceneReply {
            seq: 2,
            head: Some(b"foob".to_vec()),
            layers: vec![
                SceneLayer {
                    key: 0,
                    hash: 0x3c9e,
                    body: None,
                    patch: Some(b"fo".to_vec()),
                },
                SceneLayer {
                    key: 1,
                    hash: u64::MAX,
                    body: Some(b"f".to_vec()),
                    patch: None,
                },
                SceneLayer {
                    key: 2,
                    hash: 1,
                    body: None,
                    patch: None,
                },
            ],
            bounds: Some(vec![bound("todo.input", 1.5)]),
        }),
    };
    let line = encode_server(&frame);
    assert_eq!(
        line,
        r#"{"v":1,"id":"9","type":"scene","seq":2,"head":"Zm9vYg==","layers":[{"key":0,"hash":"0000000000003c9e","patch":"Zm8="},{"key":1,"hash":"ffffffffffffffff","body":"Zg=="},{"key":2,"hash":"0000000000000001"}],"bounds":[{"id":"todo.input","rect":[1.5,2.0,30.0,40.0]}]}"#
    );
    assert_eq!(decode_server(&line).unwrap(), frame);

    let blob = ServerMsg {
        id: Some("9".into()),
        reply: Reply::SceneBlob(BlobReply {
            key: 0xa5,
            at: 393216,
            total: 400000,
            bytes: b"foobar".to_vec(),
        }),
    };
    let line = encode_server(&blob);
    assert_eq!(
        line,
        r#"{"v":1,"id":"9","type":"scene_blob","key":"00000000000000a5","at":393216,"total":400000,"bytes":"Zm9vYmFy"}"#
    );
    assert_eq!(decode_server(&line).unwrap(), blob);
}

#[test]
fn a_frame_with_broken_bytes_does_not_read() {
    for line in [
        r#"{"v":1,"id":"9","type":"scene","seq":1,"layers":[{"key":0,"hash":"3c9e","body":"Zg=="}]}"#,
        r#"{"v":1,"id":"9","type":"scene","seq":1,"layers":[{"key":0,"hash":"000000000000zz9e"}]}"#,
        r#"{"v":1,"id":"9","type":"scene","seq":1,"layers":[{"key":0,"hash":"0000000000003c9e","body":"Zg="}]}"#,
        r#"{"v":1,"id":"9","type":"scene_blob","key":"00000000000000a5","at":0,"total":1,"bytes":"*"}"#,
    ] {
        let refused = decode_server(line).unwrap_err();
        assert!(
            matches!(refused.error, ProtoError::BadMessage { .. }),
            "{line}"
        );
    }
}

#[test]
fn a_watch_gets_the_whole_frame_then_only_what_moved() {
    let mut s = shared();
    assert!(!s.watching("main"));
    watch(&mut s, 1, "main");
    assert!(s.watching("main"));
    assert!(!s.watching("page"));

    let first = s.offer_scene("main", &offer());
    assert_eq!(blobs(&first).len(), 1, "the asset, before the frame");
    assert!(matches!(first.last().unwrap().msg.reply, Reply::Scene(_)));
    let f = frames(&first)[0];
    assert_eq!(f.seq, 1);
    assert!(f.head.is_some() && f.bounds.is_some());
    assert!(f.layers.iter().all(|l| l.body.is_some()), "whole");
    assert_eq!(
        f.layers.iter().map(|l| l.key).collect::<Vec<_>>(),
        [2, 0, 1],
        "in composite order"
    );
    assert!(first.iter().all(|o| o.conn == 1));
    assert!(first.iter().all(|o| o.msg.id.as_deref() == Some("w1")));

    assert!(s.offer_scene("main", &offer()).is_empty(), "nothing moved");
    assert!(s.offer_scene("page", &offer()).is_empty(), "no watch there");

    // layer 0 moves with a patch against what the watch holds, the
    // control moves
    let mut next = offer();
    next.layers[1] = OfferLayer {
        patch: Some((10, bytes(12, 3))),
        ..layer(0, 11, 300)
    };
    next.bounds = vec![bound("todo.input", 9.0)];
    let second = s.offer_scene("main", &next);
    assert!(blobs(&second).is_empty(), "the asset is held");
    let f = frames(&second)[0];
    assert_eq!(f.seq, 2);
    assert_eq!(f.head, None, "the head did not change");
    assert_eq!(f.bounds.as_deref(), Some(&next.bounds[..]));
    let sent: Vec<(u32, bool, bool)> = f
        .layers
        .iter()
        .map(|l| (l.key, l.body.is_some(), l.patch.is_some()))
        .collect();
    assert_eq!(
        sent,
        [(2, false, false), (0, false, true), (1, false, false)]
    );

    // a patch against a base the watch does not hold is not used
    let mut third = next.clone();
    third.layers[1] = OfferLayer {
        patch: Some((999, bytes(12, 4))),
        ..layer(0, 12, 300)
    };
    let f = s.offer_scene("main", &third);
    let f = frames(&f)[0];
    assert!(f.layers[1].body.is_some() && f.layers[1].patch.is_none());

    // a layer goes: the list says so, nothing else travels
    let mut fourth = third.clone();
    fourth.layers.remove(0);
    let out = s.offer_scene("main", &fourth);
    let f = frames(&out)[0];
    assert!(
        f.layers
            .iter()
            .all(|l| l.body.is_none() && l.patch.is_none())
    );
    assert_eq!(f.layers.len(), 2);
}

#[test]
fn an_asset_goes_once_while_the_frames_reference_it() {
    let mut s = shared();
    watch(&mut s, 1, "main");
    assert_eq!(blobs(&s.offer_scene("main", &offer())).len(), 1);
    let mut again = offer();
    again.layers[0] = layer(2, 21, 40);
    assert!(blobs(&s.offer_scene("main", &again)).is_empty());
    // a frame without it lets it go; the next that draws it sends it
    let mut without = again.clone();
    without.layers[0] = layer(2, 22, 40);
    without.assets.clear();
    assert!(blobs(&s.offer_scene("main", &without)).is_empty());
    let back = s.offer_scene("main", &again);
    assert_eq!(blobs(&back).len(), 1);
    assert_eq!(blobs(&back)[0].key, 0xa5);

    // a second watch, later, gets the whole frame and the asset
    watch(&mut s, 2, "main");
    let out = s.offer_scene("main", &again);
    assert!(out.iter().all(|o| o.conn == 2), "the first holds it all");
    assert_eq!(blobs(&out).len(), 1);
    assert!(frames(&out)[0].layers.iter().all(|l| l.body.is_some()));
}

#[test]
fn a_large_asset_and_a_large_body_go_in_parts_under_the_line_limit() {
    let mut s = shared();
    watch(&mut s, 1, "main");
    let camera = 640 * 360 * 4;
    let mut big = offer();
    big.assets = vec![(0xca, bytes(camera + 29, 9))];
    big.layers[1] = layer(0, 10, MAX_LINE);
    let out = s.offer_scene("main", &big);
    lines(&out);
    let parts = blobs(&out);
    let expect = (camera + 29).div_ceil(SCENE_PART) + MAX_LINE.div_ceil(SCENE_PART);
    assert_eq!(parts.len(), expect);
    let f = frames(&out)[0];
    assert_eq!(f.layers[1].body, None, "the body went before, as a blob");

    let mut r = SceneReader::new();
    let mut got = None;
    for o in out {
        got = r.take(o.msg.reply).unwrap().or(got);
    }
    let got = got.expect("a frame");
    assert_eq!(got.assets.len(), 1);
    assert_eq!(got.assets[0].0, 0xca);
    assert_eq!(&got.assets[0].1[..], &big.assets[0].1[..]);
    assert_eq!(
        got.layers[1].body,
        LayerBody::Body(big.layers[1].body.to_vec())
    );
    assert_eq!(
        got.layers[0].body,
        LayerBody::Body(big.layers[0].body.to_vec())
    );
}

#[test]
fn a_watch_is_one_per_connection_of_a_named_window_and_ends_with_it() {
    let mut s = shared();
    let mut host = Still::default();
    hello(&mut s, 1, "claude");
    let ask = |window: &str| ClientMsg {
        id: "x".into(),
        ask: Ask::Scene(SceneAsk {
            window: window.into(),
        }),
    };
    for bad in ["", "a b", "main!", &"w".repeat(33)] {
        let out = s.handle(1, ask(bad), &mut host);
        assert!(
            matches!(
                out[0].msg.reply,
                Reply::Error(ProtoError::BadMessage { .. })
            ),
            "{bad:?}"
        );
    }
    assert!(s.handle(1, ask("page"), &mut host).is_empty());
    let out = s.handle(1, ask("main"), &mut host);
    match &out[0].msg.reply {
        Reply::Error(ProtoError::BadMessage { reason }) => {
            assert!(reason.contains("already watches"), "{reason}");
        }
        other => panic!("{other:?}"),
    }
    assert!(s.offer_scene("main", &offer()).is_empty());
    assert_eq!(frames(&s.offer_scene("page", &offer())).len(), 1);
    s.close(1);
    assert!(!s.watching("page"));
    assert!(s.offer_scene("page", &offer()).is_empty());
}

#[test]
fn the_reader_refuses_parts_out_of_order() {
    let part = |key: u64, at: u64, total: u64, n: usize| {
        Reply::SceneBlob(BlobReply {
            key,
            at,
            total,
            bytes: vec![1; n],
        })
    };
    let frame = Reply::Scene(SceneReply {
        seq: 1,
        head: None,
        layers: Vec::new(),
        bounds: None,
    });
    let mut r = SceneReader::new();
    assert!(r.take(part(1, 4, 8, 4)).is_err(), "a part past the start");
    let mut r = SceneReader::new();
    r.take(part(1, 0, 8, 4)).unwrap();
    assert!(r.take(part(2, 0, 8, 4)).is_err(), "another blob cuts it");
    let mut r = SceneReader::new();
    r.take(part(1, 0, 8, 4)).unwrap();
    assert!(r.take(frame.clone()).is_err(), "a frame inside a blob");
    let mut r = SceneReader::new();
    assert!(r.take(part(1, 0, 8, 9)).is_err(), "past its total");
    let mut r = SceneReader::new();
    assert!(
        r.take(part(1, 0, MAX_BLOB + 1, 1)).is_err(),
        "past the most"
    );
    let mut r = SceneReader::new();
    r.take(part(1, 0, 8, 4)).unwrap();
    r.take(part(1, 4, 8, 4)).unwrap();
    let f = r.take(frame).unwrap().unwrap();
    assert_eq!(f.assets, [(1, vec![1; 8])]);
    let other = Reply::Error(ProtoError::NotEnabled { reason: "x".into() });
    assert_eq!(SceneReader::new().take(other).unwrap(), None);
}

/// a port for the toy in `dir`, and the channel its wake sends on.
fn open(dir: &Path) -> (AgentPort, Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    let config = Config {
        app: "toy".into(),
        dir: dir.to_owned(),
        ring: 8,
    };
    let port = AgentPort::open_with(config, move || {
        let _ = tx.send(());
    })
    .unwrap();
    (port, rx)
}

/// over the socket, beside an observation on the same connection: the
/// watch gets the whole frame the app holds as soon as it asks, then
/// the frames it draws, with nothing when nothing moved.
#[test]
fn a_watch_over_the_socket_beside_an_observation() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    port.share_scenes(true);
    let mut host = Still::default();
    let path = port.path().to_owned();
    let (done_tx, done) = mpsc::channel::<Vec<u64>>();
    let watcher = thread::spawn(move || {
        let mut c = Client::connect(&path).unwrap();
        c.set_timeout(Some(10_000)).unwrap();
        c.hello("viewer").unwrap();
        let id = c.watch_scene("main").unwrap();
        let look = c.send(Ask::Observe(Observe::default())).unwrap();
        let mut r = SceneReader::new();
        let mut seqs = Vec::new();
        let mut observed = false;
        while seqs.len() < 2 || !observed {
            let msg = c.recv().unwrap();
            if msg.id.as_deref() == Some(look.as_str()) {
                assert!(matches!(msg.reply, Reply::Snapshot(_)), "{msg:?}");
                observed = true;
                continue;
            }
            assert_eq!(msg.id.as_deref(), Some(id.as_str()));
            if let Some(f) = r.take(msg.reply).unwrap() {
                seqs.push(f.seq);
            }
        }
        done_tx.send(seqs).unwrap();
    });
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut frame = offer();
    let mut offered = 0;
    let seqs = loop {
        assert!(Instant::now() < deadline, "the watcher did not finish");
        let _ = woke.recv_timeout(Duration::from_millis(5));
        port.drain(&mut host);
        if port.watching("main") {
            // what the app holds, after the drain; then a frame drawn,
            // with one layer moved, then one where nothing moved
            if offered < 3 {
                if offered == 1 {
                    frame.layers[0] = layer(2, 23, 40);
                }
                port.offer_scene("main", &frame);
                offered += 1;
            }
        }
        if let Ok(seqs) = done.try_recv() {
            break seqs;
        }
    };
    watcher.join().unwrap();
    assert_eq!(seqs, [1, 2]);
    assert!(port.traffic().bytes_out > 600, "the asset went");
}

/// a watcher that does not read: the app offers frames with a new
/// camera-sized asset each and never waits; once the lines waiting for
/// the watcher pass `SCENE_QUEUE` bytes it is cut off and its watch ends.
#[test]
fn a_watcher_that_does_not_read_is_cut_and_the_app_never_waits() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    port.share_scenes(true);
    let mut host = Still::default();
    let mut c = Client::connect(port.path()).unwrap();
    c.send(Ask::Hello {
        name: "viewer".into(),
    })
    .unwrap();
    c.watch_scene("main").unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !port.watching("main") {
        assert!(Instant::now() < deadline, "the watch did not arrive");
        let _ = woke.recv_timeout(Duration::from_millis(5));
        port.drain(&mut host);
    }
    let camera = 640 * 360 * 4;
    let mut slowest = Duration::ZERO;
    let mut offered = 0usize;
    while port.watching("main") {
        assert!(offered < 64, "never cut");
        let mut frame = offer();
        frame.layers[0] = layer(2, 100 + offered as u64, 40);
        frame.assets = vec![(0xc000 + offered as u64, bytes(camera, offered as u8))];
        let at = Instant::now();
        port.offer_scene("main", &frame);
        slowest = slowest.max(at.elapsed());
        offered += 1;
    }
    let line = camera.div_ceil(3) * 4;
    assert!(
        offered * line >= SCENE_QUEUE / 2 && offered * line <= SCENE_QUEUE * 2,
        "cut after {offered} frames of {line} bytes"
    );
    assert!(
        slowest < Duration::from_secs(2),
        "an offer took {slowest:?}"
    );
    let ended = port.take_scene_watches();
    assert_eq!(
        ended.last().map(|w| w.began),
        Some(false),
        "the cut ends it"
    );
    drop(c);
}

/// with the scenes not shared (no `PLEV_MIRROR=1`), a `scene` request is
/// refused with its own code, saying what the person does to share them;
/// the rest of the protocol is as before, on the same connection.
#[test]
fn an_unshared_scene_is_refused_with_its_own_code() {
    let mut s = Server::new("toy", 8);
    let mut host = Still::default();
    hello(&mut s, 1, "claude");
    let out = s.handle(
        1,
        ClientMsg {
            id: "9".into(),
            ask: Ask::Scene(SceneAsk {
                window: "main".into(),
            }),
        },
        &mut host,
    );
    let Reply::Error(ProtoError::SceneNotShared { reason }) = &out[0].msg.reply else {
        panic!("{out:?}");
    };
    assert!(reason.contains("PLEV_MIRROR=1"), "{reason}");
    let line = encode_server(&out[0].msg);
    assert!(
        line.starts_with(r#"{"v":1,"id":"9","type":"error","error":"scene_not_shared","reason":""#),
        "{line}"
    );
    assert!(!s.watching("main"));
    assert!(s.take_scene_watches().is_empty());
    assert!(host.observed.is_empty(), "a refused watch observes nothing");
    let seen = s.handle(
        1,
        ClientMsg {
            id: "10".into(),
            ask: Ask::Observe(Observe::default()),
        },
        &mut host,
    );
    assert!(matches!(seen[0].msg.reply, Reply::Snapshot(_)));

    // over the socket: the refusal as a client reads it
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    let path = port.path().to_owned();
    let asker = thread::spawn(move || {
        let mut c = Client::connect(&path).unwrap();
        c.set_timeout(Some(10_000)).unwrap();
        c.hello("claude").unwrap();
        let id = c.watch_scene("main").unwrap();
        c.recv_for(&id).unwrap()
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    while !asker.is_finished() {
        assert!(Instant::now() < deadline, "no answer");
        let _ = woke.recv_timeout(Duration::from_millis(5));
        port.drain(&mut Still::default());
    }
    assert!(matches!(
        asker.join().unwrap(),
        Reply::Error(ProtoError::SceneNotShared { .. })
    ));
}

/// a watch is under a name: before `hello` it is refused; after it, the
/// app is told who began watching which window and when it ended, and
/// the host hears it as an observation of its actor.
#[test]
fn a_watch_has_a_name_and_the_app_is_told_of_it() {
    let mut s = shared();
    let mut host = Still::default();
    let ask = ClientMsg {
        id: "9".into(),
        ask: Ask::Scene(SceneAsk {
            window: "page".into(),
        }),
    };
    let out = s.handle(1, ask.clone(), &mut host);
    assert!(
        matches!(&out[0].msg.reply, Reply::Error(ProtoError::NotEnabled { reason }) if reason.contains("hello")),
        "{out:?}"
    );
    hello(&mut s, 1, "claude");
    assert!(s.handle(1, ask, &mut host).is_empty());
    let claude = Actor::Agent {
        name: "claude".into(),
    };
    assert_eq!(
        s.take_scene_watches(),
        [wire::SceneWatch {
            actor: claude.clone(),
            window: "page".into(),
            began: true,
        }]
    );
    let now = host.project().unwrap().revision();
    assert_eq!(host.observed, [(claude.clone(), None, now)]);
    assert!(s.take_scene_watches().is_empty(), "taken once");
    s.close(1);
    assert_eq!(
        s.take_scene_watches(),
        [wire::SceneWatch {
            actor: claude,
            window: "page".into(),
            began: false,
        }]
    );
}
