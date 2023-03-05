use encoding_rs;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rayon::prelude::*;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

const EIJIRO: &str = "EIJIRO-1448.TXT";
const EIJIROGZIP: &str = "EIJIRO-1448.tsv.gz";
const EDICT: &'static str = include_str!("../eiji-dict/edict.tab");

struct TextAppender {
    text: String,
    prev_title: Option<String>,
}

impl TextAppender {
    pub fn new(approx_fsize: usize) -> TextAppender {
        TextAppender {
            text: String::with_capacity(approx_fsize),
            prev_title: None,
        }
    }
    pub fn append(&mut self, title: &str, content: &str) {
        if content.starts_with("<→") && content.ends_with(">") {
            return;
        }
        self.prev_title = match title.find("  {") {
            Some(v) => {
                let actual_title = &title[..v];
                let attr = &title[v + 2..];
                self.write_moving_attribute(actual_title, attr, content);
                Some(actual_title.to_string())
            }
            None => {
                self.write_new_whole_line(title, content);
                None
            }
        }
    }
    fn write_new_whole_line(&mut self, title: &str, content: &str) {
        self.text.push_str("\n");
        self.text.push_str(&title);
        self.text.push_str("\t");
        self.text.push_str(&content.replace("■", "\\n"));
    }
    fn write_moving_attribute(&mut self, actual_title: &str, attr: &str, content: &str) {
        match &self.prev_title {
            Some(prev) if prev == actual_title => {
                self.text.push_str("\\n");
            }
            _ => {
                self.text.push_str("\n");
                self.text.push_str(&actual_title);
                self.text.push_str("\t");
            }
        }
        self.text
            .push_str(&attr.replace("{", "【").replace("}", "】"));
        self.text.push_str(&content.replace("■", "\\n"));
    }
    fn text_as_bytes(&self) -> &[u8] {
        self.text[1..].as_bytes()
    }
}

fn setup_eijiro() {
    println!("converting the format of {}", EIJIRO);
    let sjis = fs::read(EIJIRO).unwrap();
    let (utf8, _, _) = encoding_rs::SHIFT_JIS.decode(&sjis);
    let mut buf = TextAppender::new(utf8.len());
    for line in utf8.lines() {
        let separator = line.find(" : ").unwrap();
        buf.append(&line[3..separator], &line[separator + 3..]);
    }
    let gzip = std::fs::File::create(EIJIROGZIP)
        .expect(format!("ERROR unable to create {}", EIJIROGZIP).as_str());
    let mut w = GzEncoder::new(gzip, Compression::default());
    w.write_all(buf.text_as_bytes())
        .expect(format!("ERROR failed to write data into {}", EIJIROGZIP).as_str());
    w.flush().unwrap();
    println!("conversion finished successfully.");
}

fn edict_eijiro() -> Vec<String> {
    let gzip_path = Path::new(EIJIROGZIP);
    let source_path = Path::new(EIJIRO);
    println!("{} exists:{}", EIJIROGZIP, gzip_path.exists());
    println!("{} exists:{}", EIJIRO, source_path.exists());
    if source_path.exists() && !gzip_path.exists() {
        setup_eijiro();
    }
    if !gzip_path.exists() {
        return EDICT.lines().map(String::from).collect();
    }
    let r = std::fs::File::open(gzip_path).unwrap();
    let mut decoder = GzDecoder::new(r);
    let mut s = EDICT.to_string();
    decoder.read_to_string(&mut s).unwrap();
    s.lines().map(String::from).collect::<Vec<String>>()
}

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
    let subtitles = include_str!("../eiji-dict/train")
        .lines()
        .map(String::from)
        .collect::<Vec<String>>();

    let dicts = edict_eijiro();

    println!("\x1b[0m\x1b[1;32m検索文字\x1b[0m↵で検索");
    println!("\x1b[1;33mq\x1b[0mで次の辞書");
    println!("\x1b[1;36mctrl+c\x1b[0mでソフトウェアを終了");

    loop {
        let input: String = get_input("");
        if input.trim().is_empty() {
            continue;
        }

        for content in &[&dicts, &subtitles] {
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
