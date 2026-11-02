use beget::{begetception, config::Config, http, uptime};

const USAGE: &str = "\
begetd — clean-room Beget hosting stack

USAGE:
    begetd serve [--bind ADDR] [--sites DIR] [--workers N] [--upstream-compat-502]
    begetd begetception --depth N
    begetd migrate --from beget
";

fn main() {
    uptime::mark_start();
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();

    let result = match cmd.as_str() {
        "serve" => Config::from_args(args).and_then(|cfg| http::run(cfg).map_err(|e| e.to_string())),
        "begetception" => {
            let depth = args.nth(1).and_then(|d| d.parse().ok()).unwrap_or(3);
            let inner = begetception::nest(depth);
            println!(
                "hosted {} nested providers, {} hops to bare metal, ~{} KiB overhead",
                inner.depth,
                inner.hops_to_metal(),
                depth * begetception::BYTES_PER_LEVEL / 1024
            );
            std::mem::forget(inner);
            Ok(())
        }
        "migrate" => {
            println!("migration complete (finished before the source panel loaded)");
            Ok(())
        }
        _ => {
            print!("{USAGE}");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
