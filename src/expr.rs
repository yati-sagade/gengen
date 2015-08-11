
use std::collections::HashMap;

#[derive(Debug,Eq,Ord,PartialEq,PartialOrd,Hash,Clone)]
pub enum Token {
    Number(i64),
    Operator(String),
    LParen,
    RParen,
    Unknown,
}


pub struct Eval {
    precmap: HashMap<Token, u8>,
}

impl Eval {
    pub fn new() -> Eval {
        let mut precmap = HashMap::new();
        precmap.insert(Token::Operator("+".to_string()), 0);
        precmap.insert(Token::Operator("-".to_string()), 0);
        precmap.insert(Token::Operator("*".to_string()), 1);
        precmap.insert(Token::Operator("/".to_string()), 1);
        Eval { precmap: precmap }
    }

    pub fn to_postfix(&self, expr: &str) -> Option<Vec<Token>> {
        let string_toks: Vec<_> = expr.split(char::is_whitespace).collect();
        let tokens: Vec<Token> = string_toks.iter().map(|x: &&str| -> Token {
            match *x {
                "+" | "-" | "*" | "/" => Token::Operator(x.to_string()),
                "(" => Token::LParen,
                ")" => Token::RParen,
                s   => if s.len() > 0 && s.chars().next().unwrap().is_digit(10) {
                           Token::Number(x.parse::<i64>().unwrap())
                       } else {
                           Token::Unknown
                       }
            }
        }).collect();


        let mut stack = Vec::new();
        let mut ret = Vec::new();

        let mut number_or_not_operator = false;
        for token in &tokens {
            match *token {
                Token::LParen => {
                    stack.push((*token).clone());
                },
                Token::RParen => {
                    loop {
                        let top = match stack.pop(){
                            Some(top) => top,
                            None => { return None; }
                        };

                        match top {
                            Token::LParen => break,
                            _      => ret.push(top),
                        }
                    }
                },
                Token::Operator(ref opstr) if number_or_not_operator => {
                    number_or_not_operator = !number_or_not_operator;
                    let prec = self.precmap.get(token).unwrap();
                    loop {
                        if stack.len() == 0 {
                            break;
                        }
                        {
                            let top = match stack.last() {
                                Some(top) => top,
                                None => { return None; }
                            };

                            match top {
                                &Token::Operator(_) => {
                                    if self.precmap.get(top).unwrap() < prec {
                                        break;
                                    }
                                },
                                _ => break
                            }
                        }
                        match stack.pop() {
                            Some(x) => ret.push(x),
                            None => { return None; }
                        }
                    }
                    stack.push((*token).clone());
                },
                Token::Number(_) if !number_or_not_operator => {
                    number_or_not_operator = !number_or_not_operator;
                    ret.push((*token).clone());
                },
                Token::Unknown => { return None; },
                _ => { return None; }
            }
        }
        loop {
            if stack.len() == 0 {
                break;
            }
            match stack.pop() {
                Some(Token::LParen) => { },
                Some(t) => ret.push(t),
                None => { return None; }
            }
        }
        Some(ret)
    }

    pub fn eval(&self, expr: &str) -> Option<i64> {
        let postfix = match self.to_postfix(expr) {
            Some(postfix) => postfix,
            None => { return None; }
        };

        let mut stack: Vec<i64> = Vec::new();
        for token in &postfix {
            match *token {
                Token::Operator(ref op) => {
                    let y = match stack.pop() {
                        Some(y) => y,
                        None => { return None; }
                    };
                    let x = match stack.pop() {
                        Some(x) => x,
                        None => { return None; }
                    };
                    let s: &str = &(*op);
                    let result = match s {
                        "+" => x + y,
                        "-" => x - y,
                        "/" => {
                            if y == 0 {
                                return None;
                            } else {
                                x / y
                            }
                        },
                        "*" => x * y,
                        _   => 0,
                    };
                    stack.push(result);
                },
                Token::Number(n) => {
                    stack.push(n);
                },
                _ => { }
            }
        }
        let result = match stack.pop() {
            Some(result) => result,
            None => { return None; }
        };

        if stack.len() != 0 {
            None
        } else {
            Some(result)
        }
    }
}


