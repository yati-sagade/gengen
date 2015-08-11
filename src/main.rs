extern crate gengen;
use std::env;

use gengen::ga;
use gengen::expr::Eval;


fn main() {
    let args: Vec<_> = env::args().collect();

    for arg in args.iter().skip(1) {
        let num = match arg.parse::<i64>() {
            Ok(num) => num,
            Err(_)  => {
                println!("Could not parse \"{}\" as a number.", arg);
                continue;
            }
        };
        println!("\nGoing to try {}", num);
        match ga(num) {
            (ngens, Some(chromosome)) => {
                println!("Found solution in {} generations: {}", ngens, chromosome.decode());
            },
            (ngens, None) => {
                println!("Could not find a solution after {} generations; Giving up", ngens);
            }
        }
    }
}
