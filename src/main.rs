#[macro_use]
extern crate structopt;
extern crate atty;

use structopt::StructOpt;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs;
use std::io;
use std::cmp::max;
use atty::Stream;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(short = "h", long = "header_file")]
    header_file : Option<String>,

    #[structopt(short = "f", long = "input")]
    input_file : Option<String>,

    #[structopt(short = "c", long = "columns")]
    output_columns : Option<String>,

    #[structopt(short = "l", long = "list_header")]
    list_header : bool
}

fn infer_splitor(line : &str) -> Option<String>{
    if let Some(_) = line.find(',') {
        Some(String::from(","))
    }
    else if let Some(_) = line.find('\t') {
        Some(String::from("\t"))
    }
    else {
        None
    }
}

fn infer_header_from_file(input_reader : &mut BufRead) -> Vec<String> {
    let mut first_line = String::new();
    input_reader.read_line(&mut first_line).unwrap();
    let first_line = first_line.trim_right();

    let splitor = infer_splitor(&first_line);

    if let Some(splitor) = splitor {
        first_line.split(&splitor).map(|s| s.to_string()).collect::<Vec<String>>()
    } else {
        vec![]
    }
}

struct OutputColumn {
    name : String,
    pos : usize
}

fn main() {
    let opt = Opt::from_args();

    if opt.input_file.is_none() && atty::is(Stream::Stdin) {
        println!("missing input file or input stream");
        return;
    }

    let mut input_reader : Box<BufRead> =
        if opt.input_file.is_none() {
            Box::new(BufReader::new(io::stdin()))
        } else {
            Box::new(BufReader::new(fs::File::open(opt.input_file.unwrap()).unwrap()))
        };

    let headers_vec : Vec<String> =
        if let Some(header_file) = opt.header_file {
            let mut header_reader = BufReader::new(fs::File::open(header_file).unwrap());
            infer_header_from_file(&mut header_reader)
        }
        else {
            infer_header_from_file(&mut input_reader)
        };

    if headers_vec.len() == 0 {
        println!("cannot infer the csv header");
        return;
    }

    if opt.list_header {
        println!("{}", headers_vec.join("\n"));
        return;
    }

    let mut headers_map : HashMap<String, usize> = HashMap::new();
    for (i, header_name) in headers_vec.iter().enumerate() {
        headers_map.insert(header_name.to_string(), i);
    }

    let mut output_columns : Vec<OutputColumn> = vec![];
    let mut max_output_pos : usize = 0;

    // println!("-c is provieded? {}", opt.output_columns.is_some());

    if let Some(output_column_names) = opt.output_columns {
        for column_name in output_column_names.split(",") {
            // println!("-c wanted column {}", column_name);
            if let Some(pos) = headers_map.get(column_name) {
                output_columns.push(OutputColumn {name : column_name.to_string(), pos : *pos});
                max_output_pos = max(max_output_pos, *pos);
            }
            else
            {
                // println!("file missing column: {}", column_name);
                return;
            }
        }
    }

    let mut splitor : Option<String> = None;
    let mut is_first_content_row = true;
    for line in input_reader.lines() {
        let line = line.unwrap();
        if splitor.is_none() {
            splitor = infer_splitor(&line);
        }
        if splitor.is_none() {
            println!("error: cannot infer a proper splitor");
        }

        let splitor_str = splitor.as_ref().unwrap().clone();
        if is_first_content_row {
            let header_row = if output_columns.len() == 0 {
                headers_vec.join(&splitor_str)
            } else {
                output_columns.iter()
                    .map(|c| c.name.clone())
                    .collect::<Vec<String>>()
                    .join(&splitor_str)
            };
            println!("{}",&header_row);
        }
        is_first_content_row = false;

        let content_row = if output_columns.len() > 0 {
            let toks : Vec<&str> = line.trim_right().split(&splitor_str).collect();
            let mut output_toks: Vec<&str> = vec![];
            for col in output_columns.iter() {
                output_toks.push(toks[col.pos]);
            }
            output_toks.join(&splitor_str)
        } else {
            line
        };
        println!("{}",&content_row);
    }
}
