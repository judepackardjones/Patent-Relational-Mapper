#![allow(non_snake_case)]
use chrono::{Datelike, Duration as dur, NaiveDate};
use crossterm::event::KeyCode;
use crossterm::event::{poll, read, Event};
use regex::Regex;
use reqwest;
use reqwest::Response;
use std::fmt::Write;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Write as writ;
use std::result;
use std::thread;
use std::time::Duration;
struct Patent {
    title: String,
    date: String,
    number: i64,
}

#[tokio::main]
async fn main() -> result::Result<(), std::io::Error> {
    let mut highest: i64 = 0;
    let farming_words1: String = fs::read_to_string
    ("/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/FarmingQueryWords1.txt")
    .expect("Error reading file 1");
    //"C:\Users\judep\OneDrive\Desktop\Programming\Rust\patentRelationalMapper\Project assets\FarmingQueryWords1.txt" WINDOWS
    //"/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/FarmingQueryWords1.txt" MAC
    let farming_words_2: String = fs::read_to_string
    ("/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/FarmingQueryWords2.txt")
    .expect("Error reading file 2");
    //"C:\Users\judep\OneDrive\Desktop\Programming\Rust\patentRelationalMapper\Project assets\FarmingQueryWords2.txt" WINDOWS
    //"/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/FarmingQueryWords2.txt" MAC
    let mut farming_words: &String;
    let mut loop_counter: bool = true;
    let mut patents: Vec<Patent> = Vec::new();
    let mut patent_temp_list: Vec<Patent> = Vec::new();
    // read user key
    let (year, month, day) = regex_date(fs::read_to_string("/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/date.txt").expect("Found None"));
    let mut earliest_date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
    let mut date_text = String::new();
    let mut lowest_patent_num: i64 = read_first_line("/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/highest.txt").unwrap().parse::<i64>().unwrap();
    loop {
        println!("Earlydate at start of loop {:?}", earliest_date);
        date_text.clear();
        write!(
            date_text,
            "{:02}-{:02}-{:02}",
            earliest_date.year(),
            earliest_date.month(),
            earliest_date.day()
        )
        .expect("");
        println!("Date text is {}", date_text);
        farming_words = if loop_counter {
            &farming_words1
        } else {
            &farming_words_2
        };
        loop_counter = !loop_counter;
        println!(
            "Lowest patent num found as {}",
            &lowest_patent_num.to_string()
        );
        if poll(Duration::from_millis(100)).unwrap() {
            // will be used to break out of loop
            match read()? {
                Event::Key(event) => {
                    if event == KeyCode::Char('w').into() {
                        break;
                    } else {
                        continue;
                    }
                }

                _ => {}
            }
        } else {
            patent_temp_list.clear(); // clears temp list so it can be used on next loop.
            let query = String::from(format!(
                r#"https://api.patentsview.org/patents/query?q={{"_and":[{{"_lt":{{"patent_date":"{date_text}"}}}},{{"_gt":{{"patent_number":"{lowest_patent_num}"}}}},{{"_text_any":{{"patent_title":"{farming_words}"}}}},{{"_text_any":{{"patent_abstract":"{farming_words}"}}}}]}}&f=["patent_title","patent_date","patent_number"]"#
            ));
            let resp: Response = reqwest::get(&query).await.unwrap(); //querys the api returns Response type
            let body: String = resp.text().await.unwrap(); // parses response to String
            println!("{}", body);
            (patent_temp_list, highest) = format_patent(body); // converts the raw String to a list of patents with the Patent type
            patents.append(&mut patent_temp_list); // adds newly formatted patents to higher list.
                                                   // for pat in &patents {
                                                   //     println!("***{}***{}***{}***", pat.title, pat.date, pat.number);
                                                   //
                                                   // }
        }
        if !loop_counter && highest > lowest_patent_num {
            lowest_patent_num = highest;
        }
        earliest_date += dur::days(1);
        thread::sleep(Duration::from_secs_f32(0.8));
    } // end of loop
    for pat in &patents {
        let _ = write_patent_data(pat);
    }
    let _ = write_to_file(date_text, "/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/date.txt");
    if highest > read_first_line("/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/highest.txt").unwrap().parse::<i64>().unwrap() {
        match write_to_file(highest.to_string(), "/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/highest.txt") {
            Ok(()) => {}
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }
    Ok(())
}
//TODO:Start of functions
fn format_patent(patents: String) -> (Vec<Patent>, i64) {
    let mut highest: i64 = 0;
    let mut parsed_patent: Vec<Patent> = Vec::new();
    let re_over = Regex::new(r#"(\{"patent_title"[^}]*\})"#).unwrap();
    let mut patent_captures = vec![];
    for (_, [pat]) in re_over.captures_iter(patents.as_str()).map(|c| c.extract()) {
        patent_captures.push(pat);
    }

    let re_each =
        Regex::new(r#""patent_title":"(.*?)","patent_date":"(.*?)","patent_number":"(.*?)""#)
            .unwrap();
    let mut captures_in_vec: Vec<String> = Vec::new();
    for e in patent_captures {
        for capture in re_each.captures_iter(e) {
            // println!("*{}", e);
            captures_in_vec.clear();
            for i in 1..4 {
                if let Some(value) = capture.get(i) {
                    captures_in_vec.push(value.as_str().to_string());
                    if i == 3 {
                        let temp_val = value.as_str().to_string().parse::<i64>().unwrap();
                        if temp_val > 0 {
                            highest = temp_val
                        }
                    }
                }
            }
        }
        parsed_patent.push(Patent {
            title: captures_in_vec.get(0).unwrap_or(&"".to_string()).clone(),
            date: captures_in_vec.get(1).unwrap_or(&"".to_string()).clone(),
            number: captures_in_vec
                .get(2)
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0),
        });
    }

    (parsed_patent, highest)
}

fn write_patent_data(patent: &Patent) -> std::io::Result<()> {
    let path: &str = "/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/Patents.csv";
    let mut file = OpenOptions::new().append(true).open(path).unwrap();
    let text_owner: String = String::new(); // If patent title contained commas, it was messing up writing to csv so needed to append quotes
    let text: String = String::from(
        text_owner
            + r#"""#
            + patent.title.clone().as_str()
            + r#"""#
            + ","
            + patent.date.clone().as_str()
            + ","
            + &patent.number.to_string().clone().as_str(),
    );
    if let Err(e) = writeln!(file, "{}", text) {
        eprintln!("Couldn't write to file: {}", e);
    }
    Ok(())
}

fn read_first_line(filepath: &str) -> std::io::Result<String> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().next().unwrap().unwrap())
}

fn write_to_file(text: String, filepath: &str) -> std::io::Result<()> {
    println!("text being written:{}", text);
    println!("Filepath:{}", filepath);
    let mut file = File::create(filepath)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

fn regex_date(date: String) -> (i32, u32, u32) {
    let re_date = Regex::new(r#"(\d{4})-(\d{2})-(\d{2})"#).unwrap();
    println!("Date being parsed:{}", date);
    let mut date_vec = vec![];
    for (_, [year, month, day]) in re_date.captures_iter(&date).map(|c| c.extract()) {
        date_vec.push(year.parse::<i32>().unwrap());
        date_vec.push(month.parse::<i32>().unwrap());
        date_vec.push(day.parse::<i32>().unwrap());
    }
    (
        date_vec[0],
        date_vec[1].try_into().unwrap(),
        date_vec[2].try_into().unwrap(),
    )
}
