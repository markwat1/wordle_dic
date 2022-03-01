use regex::Regex;
use rusqlite::params;
use rusqlite::Connection;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};
const MIN_LETTERS: usize = 4;
const MAX_LETTERS: usize = 11;

fn new_i32_vec(s: usize) -> Vec<i32> {
    let mut v = Vec::with_capacity(s);
    for _i in 0..s {
        v.push(0i32);
    }
    v
}

fn connect_db(db_name: String) -> Connection {
    let dbcon = match Connection::open(db_name) {
        Ok(c) => c,
        Err(e) => {
            panic!("DB Error {}", e);
        }
    };
    dbcon
}

fn create_table(dbcon: &Connection) {
    match dbcon.execute(
        "create table if not exists word_weight (
            id integer primary key,
            word text not null unique,
            weight integer
         )",
        [],
    ) {
        Ok(_) => println!("Create Table OK"),
        Err(e) => panic!("Create Table Error {}", e),
    };
}

fn insert_word(dbcon: &Connection, w: &String, weight: i32) {
    match dbcon.execute(
        "insert or ignore into word_weight (word,weight) values (?1,?2)",
        params![w, weight],
    ) {
        Ok(_) => (), /*println!("Insert OK {}", w),*/
        Err(e) => panic!("Insert {} {} {}", e, w, weight),
    };
}

fn calc_weight(str: &String, histgram: &HashMap<char, Vec<i32>>) -> i32 {
    let mut weight_list = HashMap::new();
    let mut pos = 0;
    for c in str.chars() {
        if c.is_alphabetic() && histgram.contains_key(&c) {
            let weight = weight_list.entry(c).or_insert(0i32);
            let h = histgram.get(&c).expect("notfound");
            *weight = h[pos] as i32;
        }
        pos += 1;
    }
    let mut weight: i32 = 0;
    //    print!("{} : ", str);
    for (_k, v) in &weight_list {
        //        print!("{} + ", v);
        weight += v;
    }
    //    println!("");
    weight
}

fn main() {
    let db_name: String = "Words".to_string();
    let db_extention: String = ".db".to_string();
    let allwords = "words.txt".to_string();
    let fs = match File::open(allwords) {
        Err(why) => panic!("Could not open {}", why),
        Ok(fs) => fs,
    };
    let mut reader = BufReader::new(fs);
    let mut line = String::new();
    let mut histgram = Vec::with_capacity(MAX_LETTERS - MIN_LETTERS + 1);
    let mut words = Vec::with_capacity(MAX_LETTERS - MIN_LETTERS + 1);
    for _i in MIN_LETTERS..=MAX_LETTERS {
        histgram.push(HashMap::new());
        words.push(Vec::new());
    }
    let is_alpha = Regex::new(r"^[0-9a-z]+$").unwrap();
    while reader.read_line(&mut line).expect("read fail") > 0 {
        let l = line.to_lowercase().trim().to_string().clone();
        let length = l.len();
        if length < MIN_LETTERS || length > MAX_LETTERS || is_alpha.is_match(&l) == false {
            line.clear();
            continue;
        }
        let mut pos = 0;
        for c in l.chars() {
            if c.is_alphabetic() {
                let count = histgram[length - MIN_LETTERS]
                    .entry(c)
                    .or_insert(new_i32_vec(length));
                count[pos] += 1;
                pos += 1;
            }
        }
        words[length - MIN_LETTERS].push(l);
        line.clear();
    }
    for length in MIN_LETTERS..=MAX_LETTERS {
        for (k, v) in &histgram[length - MIN_LETTERS] {
            for (i, h) in v.iter().enumerate() {
                if i == 0 {
                    print!("{} : {}", k, h);
                } else {
                    print!(",{}", h);
                }
            }
            println!("");
        }
    }
    for length in MIN_LETTERS..=MAX_LETTERS {
        let db_file = format!("{}{}{}", db_name, length, db_extention);
        println!("db_file {}", db_file);
        let dbcon = connect_db(db_file);
        create_table(&dbcon);
        for w in &words[length - MIN_LETTERS] {
            insert_word(&dbcon, w, calc_weight(w, &histgram[length - MIN_LETTERS]));
        }
    }
}
