use std::io::Write;
use std::process::{Command, Stdio};

pub fn print_to_console(input: &String, hits: Vec<String>) {
    print_results(decorate(&input, hits));
}

fn reorder<'a>(hits: &Vec<&'a str>, input: &String) -> Vec<&'a str> {
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

fn decorate(input: &String, hits: Vec<String>) -> Vec<String> {
    let high_light_left = format!(
        "\x1b[0m\x1b[1;32m{}\x1b[0m\x1b[1;36m",
        input.replace("\t", "")
    );
    let high_light_right = format!("\x1b[1;32m{}\x1b[0m", input);
    reorder(&hits.iter().map(|s| s.as_str()).collect(), &input)
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
                    .replace(input.as_str(), &high_light_right)
            )
        })
        .collect::<Vec<String>>()
}

fn print_results(results: Vec<String>) {
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
        .write_all(results.join("\n").as_bytes())
        .is_err()
    {}
    if let Err(why) = child.wait() {
        panic!("{}", why)
    }
}
