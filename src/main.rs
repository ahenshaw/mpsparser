use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashMap;
use std::f32;

#[macro_use]
extern crate nom;

type Term = (String, f32);
type TermList = Vec<Term>;

#[derive(Debug, PartialEq, Default)]
struct MPS {
    name   : String,
    rows   : Vec<(String, String)>,
    columns: Vec<(String, String, f32)>,
    rhs    : Vec<(String, String, f32)>,
    eqns   : HashMap<String, TermList>,
    etypes : HashMap<String, String>,
    rrhs   : HashMap<String, f32>,
}

impl MPS {
    fn new() -> MPS {
        MPS::default()
    }

    fn parse(&mut self, filename:&str) {
        let f = File::open(filename).unwrap();
        let file = BufReader::new(&f);
        let mut state = "IDLE".to_string();

        for line in file.lines() {
            let line = line.unwrap();
            if !line.starts_with(" ") {
                state = self.idle(&line);
            } else {
                match state.as_str() {
                    "ROWS"    => self.rows(&line),
                    "COLUMNS" => self.columns(&line),
                    "RHS"     => self.rhs(&line),
                    "BOUNDS"  => self.blank(&state),
                    "ENDATA"  => self.blank(&state),
                    _ => ()
                }
            }
        }
    }

    fn blank(&mut self, _line:&str) {
        //println!("{}", line);
    }

    fn rows(&mut self, line:&str) {
        let tokens:Vec<&str> = line.split_whitespace().collect();
        for t in tokens.chunks(2) {
            self.rows.push((t[0].to_string(), t[1].to_string()));
        }
    }

    fn columns(&mut self, line:&str) {
        let mut tokens = line.split_whitespace();
        let label = tokens.next().unwrap();
        let leftover:Vec<&str> = tokens.collect();

        for t in leftover.chunks(2) {
            self.columns.push((label.to_string(), 
                               t[0].to_string(), 
                               t[1].parse::<f32>().unwrap()));
        }
    }

    fn rhs(&mut self, line:&str) {
        let mut tokens = line.split_whitespace();
        let label = tokens.next().unwrap();
        let leftover:Vec<&str> = tokens.collect();

        for t in leftover.chunks(2) {
            self.rhs.push((label.to_string(), 
                               t[0].to_string(), 
                               t[1].parse::<f32>().unwrap()));
        }
    }

    fn idle(&mut self, line:&str) -> String {
        named!(parser<&str, &str>,
            alt!(
                tag!("NAME") | tag!("ROWS") | tag!("COLUMNS") |
                tag!("RHS")  | tag!("BOUNDS")  |tag!("ENDATA")
            )
        );
        
        let mut new_state = "".to_string();
        match parser(line) {
            Ok((extra, tag)) => {
                if tag == "NAME" {
                    new_state = "IDLE".to_string();
                    self.name = extra.trim().to_string();
                } else {
                    new_state = tag.to_string();
                };
            },
            Err(e) => println!("Error: {}", e)
        }
        new_state
    }

    fn interpret(&mut self) {
        // build equations
        for (operator, name) in &self.rows {
            self.eqns.insert(name.clone(), vec![]);
            self.etypes.insert(name.clone(), operator.clone());
        }

        for (var, name, multiplier) in &self.columns {
            self.eqns.entry(name.clone())
                .or_insert(Vec::new())
                .push((var.clone(), multiplier.clone()));
        }
        for (_, label, value) in &self.rhs {
            self.rrhs.insert(label.clone(), value.clone());
        }
    }

    fn text(&self) -> String {
        let mut optimize = String::new();
        let mut st       = String::new();
        let bounds       = String::new();

        for (label, terms) in &self.eqns {
            let etype = self.etypes.get(label).unwrap();
            let eqn   = make_eqn(terms);
            if etype == "N" {
                optimize += format!("    {}: {}\n", label, eqn).as_str();
            } else {
                let rhs = self.rrhs.get(label).unwrap_or(&0.0);
                let op = match self.etypes.get(label).unwrap_or(&"".to_string()).as_str() {
                    "E" => "=",
                    "G" => ">=",
                    "L" => "<=",
                    _ => ""
                };
                st +=  format!("    {}: {} {} {}\n", label, eqn, op, rhs).as_str();
            }
        }
        format!("Optimize\n{}Subject To\n{}Bounds\n{}End", 
                optimize, st, bounds)
    }
}

fn make_eqn(terms:&TermList) -> String {
    let mut eqn = String::new();

    for (var, mul) in terms {
        let mut sgn = if *mul < 0.0 {"-"} else {if eqn.len() > 0 {"+"} else {""}};
        let mut mult = String::new();
        if  mul.abs() != 1.0 {
            mult = format!("{}*", mul.abs()).clone();
        }
        eqn += format!(" {} {}{}", sgn, mult, var).as_str();
    }
    eqn
}

fn main() {
    let mut mps = MPS::new();
    // mps.parse("data/afiro.mps");
    mps.parse("data/wikipedia.mps");
    mps.interpret();
    println!("{}", mps.text());
}
