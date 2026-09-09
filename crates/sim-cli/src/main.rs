fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let parsed = (args.len() == 2).then(|| (args[0].parse::<u32>(), args[1].parse::<u32>()));
    match parsed {
        Some((Ok(left), Ok(right))) => println!("{}", sim_core::build_probe(left, right)),
        _ => {
            eprintln!("usage: sim-cli <u32> <u32> (build probe only)");
            std::process::exit(2);
        }
    }
}
