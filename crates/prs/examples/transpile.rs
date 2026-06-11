//! The parser's command line: react component sources in, plev builder
//! code out, droplist on stderr.
//!
//! Run: `cargo run -p prs --example transpile -- <index.tsx> <module.sass> <vars.sass>`

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [tsx, sass, vars] = args.as_slice() else {
        eprintln!("uso: transpile <index.tsx> <module.sass> <vars.sass>");
        std::process::exit(2);
    };
    let read = |p: &str| {
        std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("nao li {p}: {e}");
            std::process::exit(1);
        })
    };
    let (tsx_src, sass_src, vars_src) = (read(tsx), read(sass), read(vars));
    let name = |p: &str| {
        std::path::Path::new(p)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| p.to_string())
    };
    match prs::transpile_react((&name(tsx), &tsx_src), (&name(sass), &sass_src), &vars_src) {
        Ok(out) => {
            println!("{}", out.code);
            eprintln!(
                "-- mapeadas {} | droplist {} entradas (arquivo:linha + motivo)",
                out.mapped,
                out.dropped.len()
            );
        }
        Err(e) => {
            eprintln!("falhou: {e}");
            std::process::exit(1);
        }
    }
}
