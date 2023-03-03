use rayon::prelude::*;
use std::io;
use std::io::Write;
use std::process::{Command, Stdio};

fn reorder<'a>(hits: &Vec<&'a String>, input: String) -> Vec<&'a String> {
    let mut a = vec![];
    let mut b = vec![];
    for i in 0..hits.len() {
        if hits[i].starts_with(input.as_str()) {
            a.push(hits[i])
        } else {
            b.push(hits[i])
        }
    }
    a.append(&mut b);
    a
}

fn main() {
    let wn_content = include_str!("../eiji-dict/eiji-127.txt")
        .lines()
        .map(String::from)
        .collect::<Vec<String>>();

    let wb_content = include_str!("../eiji-dict/train")
        .lines()
        .map(String::from)
        .collect::<Vec<String>>();

    loop {
        let input: String = get_input("");
        if input.trim().is_empty() {
            continue;
        }

        for content in &[&wn_content, &wb_content] {
            let res = filter(&content, &input);
            let mut child = Command::new("less")
                .arg("-R")
                .arg("-M")
                .arg("+Gg")
                .arg("-s")
                .stdin(Stdio::piped())
                .spawn()
                .unwrap();
            if child
                .stdin
                .as_mut()
                .ok_or("Child process stdin has not been captured!")
                .unwrap()
                .write_all(res.join("\n").as_bytes())
                .is_err()
            {}

            if let Err(why) = child.wait() {
                panic!("{}", why)
            }
        }
    }
}

fn filter(content: &[String], input: &str) -> Vec<String> {
    let high_light_left = format!(
        "\x1b[0m\x1b[1;32m{}\x1b[0m\x1b[1;36m",
        input.replace("\t", "")
    );
    let high_light_right = format!("\x1b[1;32m{}\x1b[0m", input);

    let hits = content
        .par_iter()
        .filter(|l| l.contains(input))
        .collect::<Vec<&String>>();

    reorder(&hits, input.to_string())
        .iter()
        .map(|l| {
            let tabi = l.find('\t').unwrap();
            let left = &l[0..tabi];
            let right = &l[tabi + 1..];
            format!(
                "\x1b[1;36m{}\x1b[0m  {}",
                left.replace(&input.replace("\t", ""), &high_light_left),
                right
                    .replace("\\n", "\n")
                    .replace("<ħ>", "\x1b[9m")
                    .replace("</ħ>", "\x1b[0m")
                    .replace(&input, &high_light_right)
            )
        })
        .collect::<Vec<String>>()
}

fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {}
        Err(_no_updates_is_fine) => {}
    }
    input
        .replace("\t", "📙")
        .replace(" ", "🍵")
        .trim()
        .replace("🍵", " ")
        .replace("📙", "\t")
}
