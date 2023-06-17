use std::env;

use copier::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("err: Not enough arguments");
    } else {
        let config = Config::new(&args);
        println!(
            "Copying files from {} to {}",
            config.source.to_str().unwrap(),
            config.target.to_str().unwrap()
        );
        if let Some(ignore_path) = config.ignore_path {
            println!("Using ignore file {}", ignore_path.to_str().unwrap())
        }
        match copier::run(config) {
            Ok(_) => println!("Finished"),
            Err(err) => eprintln!("err: Failed to copy: {:?}", err),
        };
    }
}
