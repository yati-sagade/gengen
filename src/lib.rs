#![feature(append)]
#![feature(slice_patterns)]

extern crate rand;
extern crate num;
pub mod expr;

use expr::Eval;
use rand::Rng;
use std::cmp;

const MAX_GENS: usize = 1000;
const POP_SIZE: usize = 500;
const CHROMOSOME_MIN: usize = 3;
const CHROMOSOME_MAX: usize = 101;
const MUTATION_RATE: f64 = 0.01;
const CROSSOVER_RATE: f64 = 0.70;
const EPSILON: f64 = 10e-8;


fn randrange(lo: f64, hi: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(lo, hi)
}


pub fn from_binary(bits: &[bool]) -> usize {
    let mut acc: usize = 0;
    let mut p = bits.len() - 1;
    for bit in bits {
        if *bit {
            acc |= 0x1 << p;
        }
        if (p > 0) {
            p -= 1;
        }
    }
    acc
}


pub fn binary(x: usize) -> Vec<bool> {
    let mut ret = Vec::new();
    let mut x = x;

    loop {
        ret.push((x & 0x01) == 1);
        x >>= 1;
        if (x == 0) {
            break;
        }
    }
    ret.reverse();
    ret
}


#[derive(Clone)]
pub struct Chromosome {
    pub bits: Vec<bool>,
    pub fitness: Option<f64>,
}

impl Chromosome {
    fn new() -> Chromosome {
        Chromosome { bits: Vec::new(), fitness: Some(0f64) }
    }

    fn set_expr(&mut self, exp: &str) {
        self.bits = encode_expression(exp);
    }
    
    fn evaluate_fitness(&mut self, target: i64) {
        let eval = Eval::new();
        let decoded = self.decode();
        self.fitness = match eval.eval(&decoded) {
            Some(val) => {
                let delta = (val - target).abs() as f64;
                Some(1f64 / (1f64 + delta))
            }
            None => {
                Some(0.0)
            } 
        };
    }

    pub fn decode(&self) -> String {
        let mut i = 0;
        let mut ret = Vec::new();
        while i + 3 < self.len() {
            let val = from_binary(&vec![self.bits[i],self.bits[i+1],self.bits[i+2],self.bits[i+3]]);
            i += 4;
            if val <= 9 {
                ret.push(val.to_string());
            } else {
                match val {
                    10 => ret.push("+".to_string()),
                    11 => ret.push("-".to_string()),
                    12 => ret.push("*".to_string()),
                    13 => ret.push("/".to_string()),
                    _  => { },
                }
            }
        }
        ret.connect(" ")
    }

    pub fn correct(&mut self) {
        let decoded = self.decode();
        let dec: Vec<_> = decoded.split(char::is_whitespace).collect();
        let mut op = false;
        let mut result = Vec::new();
        let mut i = 0;
        while i + 3 < self.bits.len() {
            let val = from_binary(&vec![self.bits[i],
                                        self.bits[i+1],
                                        self.bits[i+2],
                                        self.bits[i+3]]);
            if op {
                if 10 <= val && val <= 13 {
                    result.push(get_operator(val as u8));
                    op = false;
                }
            } else {
                if val <= 9 {
                    result.push(format!("{}", val).to_string());
                    op = true;
                }
            }
            i += 4;
        }
        let connected: &str = &result.connect(" ");
        self.set_expr(connected);
    }

    fn len(&self) -> usize { self.bits.len() }

    fn mutate(&mut self) -> bool {
        let mut rng = rand::thread_rng();
        let mut mutated = false;
        for b in self.bits.iter_mut() {
            if rng.gen_range(0f64, 1f64) < 0.01 {
                *b = ! *b;
                mutated = true;
            }
        }
        mutated
    }

}

fn get_operator(val: u8) -> String {
    match val {
        10 => "+",
        11 => "-",
        12 => "*",
        13 => "/",
        x => panic!("Invalid value {}", x),
    }.to_string()
}


fn crossover(c1: &mut Chromosome, c2: &mut Chromosome) -> bool {
    let mut rng = rand::thread_rng();
    if randrange(0f64, 1f64) <= 0.7 {
        let idx: usize = rng.gen_range(0, cmp::max(c1.len(), c2.len()));
        let mut acc1: Vec<bool> = Vec::new();
        let mut acc2: Vec<bool> = Vec::new();

        for i in 0..cmp::min(c1.len(), idx) {
            acc1.push(c1.bits[i].clone());
        }

        for i in 0..cmp::min(c2.len(), idx) {
            acc2.push(c2.bits[i].clone());
        }

        for i in idx..cmp::max(c1.len(), c2.len()) {
            if i < c1.len() {
                acc2.push(c1.bits[i].clone());
            }
            if i < c2.len() {
                acc1.push(c2.bits[i].clone());
            }
        }

        c1.bits = acc1;
        c2.bits = acc2;

        let r1 = c1.bits.len() % 4;
        let r2 = c2.bits.len() % 4;

        if (r1 != 0) {
            for _ in 0..(4-r1) {
                c1.bits.push(rng.gen());
            }
        }

        if (r2 != 0) {
            for _ in 0..(4-r2) {
                c2.bits.push(rng.gen());
            }
        }

        true
    } else {
        false
    }
}

fn encode_expression(expr: &str) -> Vec<bool> {
    let tokens: Vec<_> = expr.split(char::is_whitespace).collect();     
    let mut bits = Vec::new();
    for token in tokens {
        match token {
            ""  => continue,
            "+" => bits.append(&mut vec![true, false, true, false]),
            "-" => bits.append(&mut vec![true, false, true, true]),
            "*" => bits.append(&mut vec![true, true, false, false]),
            "/" => bits.append(&mut vec![true, true, false, true]),

            num => {
                let num = match num.parse::<usize>() {
                    Ok(num) => num,
                    Err(x) => panic!("This fucker is the culprit {}", num),
                };
                let mut bin = binary(num);
                for i in bin.len()..4 {
                    bits.push(false);
                }
                bits.append(&mut bin);
            }
        }
    }
    bits
}


fn select(population: &[Chromosome], total_fitness: f64) -> Option<Chromosome> {  
    let mut rng = rand::thread_rng();
    let fitness_slice = rng.gen_range(0f64, 1f64) * total_fitness;
    let mut acc = 0f64;
    for p in population {
        acc += p.fitness.unwrap();
        if (acc >= fitness_slice) {
            return Some((*p).clone());
        }
    } 
    None
}


pub fn ga(target: i64) -> (usize, Option<Chromosome>) {
    let mut rng = rand::thread_rng();
    let mut population = Vec::new();
    let mut ngens: usize = 0;

    for i in 0..500 {
        let mut c = Chromosome::new();
        let n = rng.gen_range(CHROMOSOME_MIN, CHROMOSOME_MAX);
        c.bits = rng.gen_iter().take(n * 4).collect();
        population.push(c);
    }

    loop {
        let mut total_fitness = 0f64;
        ngens += 1;
        if ngens > MAX_GENS {
            return (ngens-1, None);
        }

        println!("Generation {}", ngens);
        for p in population.iter_mut() {
            p.evaluate_fitness(target);
            if ((p.fitness.unwrap() - 1f64).abs() <= EPSILON) {
                return (ngens, Some((*p).clone()));
            }
            total_fitness += p.fitness.unwrap();
        }
        let mut tmp = Vec::new();

        loop {
            let mut c1 = None;
            loop {
                c1 = select(&population, total_fitness);
                match c1 {
                    Some(_) => break,
                    None => { },
                }
            }

            // println!("Selected 1 {}", c1.decode());

            let mut c2 = None;
            loop {
                c2 = select(&population, total_fitness);
                match c1 {
                    Some(_) => break,
                    None => { },
                }
            }

            // println!("Selected 2 {}", c2.decode());

            let mut c1 = c1.unwrap();
            let mut c2 = c2.unwrap();

            crossover(&mut c1, &mut c2);
            c1.mutate();
            c2.mutate();

            c1.correct();
            c2.correct();

            tmp.push(c1);
            tmp.push(c2);

            if tmp.len() == 500 {
                break
            }
        }

        population = tmp;
    }
}
