//! lot conversion hot path: parse a lottie json and convert it to the
//! .monster bytes. the input is a small self-contained lottie (one shape
//! layer: a filled rect group) so the bench needs no external fixture; the
//! work measured is the serde parse, the per-frame render to paths, delta
//! discovery and the encode pipeline.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lot::convert;

const LOTTIE: &str = r#"{
  "fr": 30, "ip": 0, "op": 30, "w": 200, "h": 200,
  "layers": [
    { "ty": 4, "ind": 1, "ip": 0, "op": 30, "st": 0,
      "ks": { "p": {"a":0,"k":[100,100]}, "o": {"a":0,"k":100} },
      "shapes": [
        { "ty": "gr", "it": [
          { "ty": "rc", "p": {"a":0,"k":[0,0]}, "s": {"a":0,"k":[80,80]}, "r": {"a":0,"k":0} },
          { "ty": "fl", "c": {"a":0,"k":[1.0,0.2,0.2,1.0]}, "o": {"a":0,"k":100} },
          { "ty": "tr", "p": {"a":0,"k":[0,0]}, "a": {"a":0,"k":[0,0]},
            "s": {"a":0,"k":[100,100]}, "r": {"a":0,"k":0}, "o": {"a":0,"k":100} }
        ]}
      ]
    }
  ]
}"#;

fn bench_convert(c: &mut Criterion) {
    c.bench_function("lot_convert_shape_layer", |b| {
        b.iter(|| convert(black_box(LOTTIE), "bench").expect("convert"));
    });
}

criterion_group!(benches, bench_convert);
criterion_main!(benches);
