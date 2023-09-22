use std::result;
use std::thread;
use std::time::Duration;
use crossterm::event::KeyCode;
use reqwest;
use reqwest::Response;
use regex::{Regex, Captures};
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use crossterm::event::{poll, read, Event};

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
    let mut loop_counter: i8 = 0;
    let mut patents: Vec<Patent> = Vec::new();
    let mut patent_temp_list: Vec<Patent> = Vec::new();
    // read user key
    loop {
        let lowest_patent_num: i64 = read_first_line().unwrap(); 
        println!("Lowest patent num found as {}", &lowest_patent_num.to_string());
            if poll(Duration::from_millis(100)).unwrap() { // will be used to break out of loop
                match read()? { 
                    Event::Key(event)=>{
                        if event == KeyCode::Char('w').into(){
                            break;
                        } else {
                            continue;
                        }
                        },

                    
                    Event::FocusGained => todo!(),
                    Event::FocusLost => todo!(),
                    Event::Mouse(_) => todo!(),
                    Event::Paste(_) => todo!(),
                    Event::Resize(_, _) => todo!(), }
            }
            else {
                patent_temp_list.clear(); // clears temp list s oti can be used on next loop.
    match loop_counter { //changes the query word list every other loop 
        0 => {
            loop_counter = 1;
            farming_words = &farming_words1;
        }
        1 => {
            loop_counter = 0;
            farming_words = &farming_words_2;
        }
        _ => {
            panic!("Loop counter set to non-valid value.");
        }
    }
    let mut query = String::from(format!(r#"https://api.patentsview.org/patents/query?q={{"_and":[{{"_gt":{{"patent_number":"{lowest_patent_num}"}}}},{{"_text_any":{{"patent_title":"{farming_words}"}}}},{{"_text_any":{{"patent_abstract":"{farming_words}"}}}}]}}&f=["patent_title","patent_date","patent_number"]"#));
    let resp: Response = reqwest::get(&query).await.unwrap(); //querys the api returns Response type
    let body: String = resp.text().await.unwrap(); // parses response to String
    //println!("{}", body);
    (patent_temp_list, highest) = format_patent(body); // converts the raw String to a list of patents with the Patent type
    patents.append(&mut patent_temp_list); // adds newly formatted patents to higher list. 
    for pat in &patents {
        // println!("***{}***{}***{}***", pat.title, pat.date, pat.number);
    }
    }
    if loop_counter == 1 { 
        if highest > read_first_line().unwrap() {
            write_highest(highest);
        }
        //Change to add comparison and writing highest into csv
    }
    thread::sleep(Duration::from_secs_f32(1.3));
}
    for pat in &patents {
        let _ = write_data(pat);
    }

    Ok(())
}




fn format_patent(patents: String) -> (Vec<Patent>, i64){
    let mut highest: i64 = 0;
    let mut parsed_patent: Vec<Patent> = Vec::new();
    let re_over = Regex::new(r#"(\{"patent_title"[^}]*\})"#).unwrap();
    let mut patent_captures = vec![];
    for (_, [pat]) in re_over.captures_iter(patents.as_str()).map(|c| c.extract()) {
        patent_captures.push(pat);
    }
    
    let re_each = Regex::new(r#""patent_title":"(.*?)","patent_date":"(.*?)","patent_number":"(.*?)""#).unwrap();
    let mut captures_in_vec: Vec<String> = Vec::new();
    for e in patent_captures{
        for capture in re_each.captures_iter(e) {
            // println!("*{}", e);
            captures_in_vec.clear();
            for i in 1..4{
         if let Some(value) = capture.get(i) {
            captures_in_vec.push(value.as_str().to_string());
            if i == 3 {
                let temp_val = value.as_str().to_string().parse::<i64>().unwrap();
                if temp_val > 0 {highest = temp_val}
            }
            }
        }
    }
    parsed_patent.push(Patent {
        title: captures_in_vec.get(0).unwrap_or(&"".to_string()).clone(),
        date: captures_in_vec.get(1).unwrap_or(&"".to_string()).clone(),
        number: captures_in_vec.get(2)
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0), 
    });
    }
    
    (parsed_patent, highest)
}



fn write_data(patent: &Patent) -> std::io::Result<()> {
    let path: &str = "/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/Patents.csv";
    let mut file = OpenOptions::new()
    .append(true)
    .open(path)
    .unwrap();
    let text_owner: String = String::new(); // If patent title contained commas, it was messing up writing to csv so needed to append quotes
    let text: String = String::from(text_owner + r#"""# + patent.title.clone().as_str() + r#"""# + "," + patent.date.clone().as_str() + "," + &patent.number.to_string().as_str().clone());
    if let Err(e) = writeln!(file, "{}", text) {
        eprintln!("Couldn't write to file: {}", e);
    }
    Ok(())
}



fn read_first_line() -> std::io::Result<i64> {
    let filepath: &str = "/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/highest.txt";
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    return Ok(reader.lines().next().unwrap().unwrap().parse::<i64>().unwrap());
    
}



fn write_highest(highest: i64) -> std::io::Result<()>{
    let filepath: &str = "/Users/judepackard-jones/Desktop/Programming/Rust/Patent-relational-mapper/Project assets/highest.txt";
    // Add windows path
    let mut file = File::create(filepath)?;
    file.write_all(highest.to_string().as_bytes())?;
    Ok(())

}