#[macro_use]
extern crate log;

use encoding_rs;
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

const EDICT_NGRAM: &str = "EDICT_NGRAM";
const EDICT_INDEX: &str = "EDICT_INDEX";
const EDICT_TEXT: &str = "EDICT_TEXT";

const SUBTITLE_NGRAM: &str = "SUBTITLE_NGRAM";
const SUBTITLE_INDEX: &str = "SUBTITLE_INDEX";
const SUBTITLE_TEXT: &str = "SUBTITLE_TEXT";

const EIJIRO: &str = "EIJIRO-1448.TXT";
const EIJIRO_TEXT: &str = "EIJIRO-1448_TEXT";
const EIJIRO_NGRAM: &str = "EIJIRO-1448_NGRAM";
const EIJIRO_INDEX: &str = "EIJIRO-1448_INDEX";

const REIJIRO: &str = "REIJI-1441.TXT";
const REIJIRO_TEXT: &str = "REIJI-1441_TEXT";
const REIJIRO_NGRAM: &str = "REIJI-1441_NGRAM";
const REIJIRO_INDEX: &str = "REIJI-1441_INDEX";

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
}

const SUBTITLE: &str = "eiji-dict/train";
fn setup_subtitle() {
    println!("building the index of {}", SUBTITLE);
    let utf8 = fs::read_to_string(SUBTITLE).unwrap();
    let mut words: Vec<[u8; 20]> = Vec::with_capacity(utf8.len());
    let mut acc = 0u32;
    for line in utf8.lines().map(|v| v.to_string()) {
        add_segments(&mut words, &line, acc);
        acc += line.len() as u32;
        acc += "\n".len() as u32;
    }
    words.sort();
    words.dedup();
    write_indices(words, SUBTITLE_NGRAM, SUBTITLE_INDEX);
    fs::write(
        SUBTITLE_TEXT,
        &utf8.lines().collect::<Vec<&str>>().join("\n"),
    )
    .unwrap();
    println!("indexing finished successfully.");
}

const EDICT: &str = "eiji-dict/edict.tab";
fn setup_edict() {
    println!("building the index of {}", EDICT);
    let utf8 = fs::read_to_string(EDICT).unwrap();
    let mut words: Vec<[u8; 20]> = Vec::with_capacity(utf8.len());
    let mut acc = 0u32;
    for line in utf8.lines().map(|v| v.to_string()) {
        add_segments(&mut words, &line, acc);
        acc += line.len() as u32;
        acc += "\n".len() as u32;
    }
    words.sort();
    words.dedup();
    write_indices(words, EDICT_NGRAM, EDICT_INDEX);
    fs::write(EDICT_TEXT, &utf8.lines().collect::<Vec<&str>>().join("\n")).unwrap();
    println!("indexing finished successfully.");
}

fn setup_eijiro() {
    println!("building the index of {}", EIJIRO);
    let sjis = fs::read(EIJIRO).unwrap();
    let (utf8, _, _) = encoding_rs::SHIFT_JIS.decode(&sjis);
    let mut buf = TextAppender::new(utf8.len());
    for line in utf8.lines() {
        let separator = line.find(" : ").unwrap();
        buf.append(&line[3..separator], &line[separator + 3..]);
    }

    let mut words: Vec<[u8; 20]> = Vec::with_capacity(buf.text.len());
    let mut acc = 0u32;
    for line in buf.text[1..].lines().map(|v| v.to_string()) {
        add_segments(&mut words, &line, acc);
        acc += line.len() as u32;
        acc += "\n".len() as u32;
    }
    words.sort();
    words.dedup();
    write_indices(words, EIJIRO_NGRAM, EIJIRO_INDEX);
    fs::write(EIJIRO_TEXT, &buf.text[1..]).unwrap();
    println!("indexing finished successfully.");
}

fn setup_reijiro() {
    println!("building the index of {}", REIJIRO);
    let sjis = fs::read(REIJIRO).unwrap();
    let (utf8, _, _) = encoding_rs::SHIFT_JIS.decode(&sjis);
    let mut text = String::with_capacity(utf8.len());
    for line in utf8.lines() {
        let separator = line.find(" : ").unwrap();
        let (title, content) = (&line[3..separator], &line[separator + 3..]);
        text.push_str("\n");
        text.push_str(&title);
        text.push_str("\t");
        for (i, segment) in &mut content.replace("／", "").split("◆").enumerate() {
            if i == 0 {
                text.push_str(segment);
                continue;
            }
            if segment.starts_with("ことわざ") || segment.starts_with("金言") {
                text.push_str("◆");
                text.push_str(segment);
                continue;
            }
        }
    }

    let mut words: Vec<[u8; 20]> = Vec::with_capacity(text.len());
    let mut acc = 0u32;
    for line in text[1..].lines().map(|v| v.to_string()) {
        add_segments(&mut words, &line, acc);
        acc += line.len() as u32;
        acc += "\n".len() as u32;
    }
    words.sort();
    words.dedup();
    write_indices(words, REIJIRO_NGRAM, REIJIRO_INDEX);
    fs::write(REIJIRO_TEXT, &text[1..]).unwrap();
    println!("indexing finished successfully.");
}

fn write_indices(data: Vec<[u8; 20]>, ngram: &str, index: &str) {
    let mut ngramf = BufWriter::new(fs::File::create(ngram).unwrap());
    let mut indexf = BufWriter::new(fs::File::create(index).unwrap());
    for &value in &data {
        ngramf.write(&value[0..12]).unwrap();
        indexf.write(&value[12..20]).unwrap();
    }
}

fn add_segments(words: &mut Vec<[u8; 20]>, line: &String, offset: u32) {
    let bs = line.as_bytes();
    for p in 0..bs.len() {
        let mut u = [0; 20];
        for i in 0..12 {
            u[i] = bs.get(p + i).map(|&v| v).unwrap_or(0);
        }
        u[12] = (offset >> 24) as u8;
        u[13] = (offset >> 16) as u8;
        u[14] = (offset >> 8) as u8;
        u[15] = (offset >> 0) as u8;
        let len = line.len();
        u[16] = (len >> 24) as u8;
        u[17] = (len >> 16) as u8;
        u[18] = (len >> 8) as u8;
        u[19] = (len >> 0) as u8;
        words.push(u);
    }
}

fn check_reijiro() {
    let source_path = Path::new(REIJIRO);
    let ngram_path = Path::new(REIJIRO_NGRAM);
    let index_path = Path::new(REIJIRO_INDEX);
    let text_path = Path::new(REIJIRO_TEXT);
    println!("{} exists:{}", REIJIRO, source_path.exists());
    println!("{} exists:{}", REIJIRO_NGRAM, ngram_path.exists());
    println!("{} exists:{}", REIJIRO_INDEX, index_path.exists());
    println!("{} exists:{}", REIJIRO_TEXT, text_path.exists());
    if source_path.exists() && !text_path.exists() {
        setup_reijiro();
    }
}

fn check_eijiro() {
    let source_path = Path::new(EIJIRO);
    let ngram_path = Path::new(EIJIRO_NGRAM);
    let index_path = Path::new(EIJIRO_INDEX);
    let text_path = Path::new(EIJIRO_TEXT);
    println!("{} exists:{}", EIJIRO, source_path.exists());
    println!("{} exists:{}", EIJIRO_NGRAM, ngram_path.exists());
    println!("{} exists:{}", EIJIRO_INDEX, index_path.exists());
    println!("{} exists:{}", EIJIRO_TEXT, text_path.exists());
    if source_path.exists() && !text_path.exists() {
        setup_eijiro();
    }
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

fn main() {
    env_logger::init();
    // setup_edict();
    // setup_subtitle();
    check_eijiro();
    check_reijiro();

    println!("\x1b[0m\x1b[1;32m検索文字\x1b[0m(Enter)で検索");
    println!("\x1b[1;33md\x1b[0mで画面をスクロール \x1b[1;33mq\x1b[0mで次の辞書");
    println!("\x1b[1;36mctrl+c\x1b[0mでソフトウェアを終了");

    loop {
        let input: String = get_input("");
        if input.trim().is_empty() {
            continue;
        }

        edict_eiji(&input);
        {
            let nums = ngram_search(&input, SUBTITLE_NGRAM, SUBTITLE_INDEX);
            let hits = filter(&input, &nums, SUBTITLE_TEXT);
            print_results(decorate(&input, hits));
        }
        if Path::new(REIJIRO_TEXT).exists() {
            let nums = ngram_search(&input, REIJIRO_NGRAM, REIJIRO_INDEX);
            let hits = filter(&input, &nums, REIJIRO_TEXT);
            print_results(decorate(&input, hits));
        }
    }
}

fn edict_eiji(input: &String) {
    let nums = ngram_search(&input, EDICT_NGRAM, EDICT_INDEX);
    let mut hits = filter(&input, &nums, EDICT_TEXT);
    if Path::new(EIJIRO_TEXT).exists() {
        let nums = ngram_search(&input, EIJIRO_NGRAM, EIJIRO_INDEX);
        let mut eiji_hits = filter(&input, &nums, EIJIRO_TEXT);
        hits.append(&mut eiji_hits)
    }
    print_results(decorate(&input, hits));
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

fn filter(input: &str, nums: &Vec<(u32, u32)>, text_file: &str) -> Vec<String> {
    debug!("{:?} given:{}", nums, nums.len());
    if nums.len() == 0 {
        return vec![];
    }

    let lines = nums
        .par_iter()
        .map(|&(offset, len)| {
            let mut txtf = File::open(text_file).unwrap();
            txtf.seek(SeekFrom::Start(offset as u64)).unwrap();
            let mut line = vec![0u8; len as usize];
            txtf.read(&mut line[..]).unwrap();
            String::from_utf8(line).unwrap()
        })
        .collect::<Vec<String>>();

    lines
        .par_iter()
        .filter(|l| l.contains(input))
        .map(|s| s.to_string())
        .collect()
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

fn ngram_search(keyword: &String, ngram: &str, index: &str) -> Vec<(u32, u32)> {
    fn to_u32(b: &[u8; 4]) -> u32 {
        let mut n = 0u32;
        n |= b[0] as u32;
        n <<= 8;
        n |= b[1] as u32;
        n <<= 8;
        n |= b[2] as u32;
        n <<= 8;
        n |= b[3] as u32;
        n
    }
    if keyword.is_empty() {
        return vec![];
    }
    let c = keyword.as_bytes();
    let mut search_block = [0u8; BLOCK_SIZE as usize];
    for (i, &v) in c.iter().enumerate() {
        if i > search_block.len() - 1 {
            break;
        }
        search_block[i] = v;
    }
    let mut ngramf = File::open(ngram).unwrap();
    let begin = limit_left(&mut ngramf, &search_block) / BLOCK_SIZE * 8;
    let end = limit_right(&mut ngramf, &search_block) / BLOCK_SIZE * 8;

    let mut indexf = File::open(index).unwrap();
    let mut nums = vec![];
    for p in (begin..end).step_by(8) {
        let (mut offset, mut len) = ([0u8; 4], [0u8; 4]);
        indexf.seek(SeekFrom::Start(p)).unwrap();
        indexf.read_exact(&mut offset).unwrap();
        indexf.seek(SeekFrom::Start(p + 4)).unwrap();
        indexf.read_exact(&mut len).unwrap();
        nums.push((to_u32(&offset), to_u32(&len)));
    }
    nums.sort();
    nums.dedup();
    if nums.len() > 9999 {
        info!(
            "because it found too many hits of {}, will truncate to 9999",
            nums.len()
        );
        nums.truncate(9999);
    }
    nums
}

/// 12 byte length segmentation
const BLOCK_SIZE: u64 = 12;

fn limit_right(index: &mut File, head: &[u8; BLOCK_SIZE as usize]) -> u64 {
    let (mut word, mut next) = ([0u8; BLOCK_SIZE as usize], [0u8; BLOCK_SIZE as usize]);
    let blocks = index.metadata().unwrap().len() / BLOCK_SIZE;
    let (mut fr, mut to) = (0u64, blocks * BLOCK_SIZE);
    let mut cursor = (blocks / 2 - 1) * BLOCK_SIZE;
    loop {
        if cursor == 0 || cursor == (blocks - 1) * BLOCK_SIZE {
            return cursor + BLOCK_SIZE;
        }
        index.seek(SeekFrom::Start(cursor)).unwrap();
        index.read_exact(&mut word).unwrap();
        index.seek(SeekFrom::Start(cursor + BLOCK_SIZE)).unwrap();
        index.read_exact(&mut next).unwrap();

        for (i, &c) in head.iter().enumerate().rev() {
            if c != 0 {
                break;
            }
            (word[i], next[i]) = (0, 0);
        }

        debug!("{:?}", String::from_utf8(word.iter().map(|&v| v).collect()));
        if word <= *head && *head < next {
            return cursor + BLOCK_SIZE;
        }
        if *head < word {
            to = cursor;
            cursor -= (cursor - fr) / BLOCK_SIZE / 2 * BLOCK_SIZE;
        } else if word <= *head {
            fr = cursor;
            cursor += (to - cursor) / BLOCK_SIZE / 2 * BLOCK_SIZE;
        }
    }
}

fn limit_left(index: &mut File, head: &[u8; BLOCK_SIZE as usize]) -> u64 {
    let (mut word, mut next) = ([0u8; BLOCK_SIZE as usize], [0u8; BLOCK_SIZE as usize]);
    let blocks = index.metadata().unwrap().len() / BLOCK_SIZE;
    let (mut fr, mut to) = (0u64, blocks * BLOCK_SIZE);
    let mut cursor = (blocks / 2 - 1) * BLOCK_SIZE;
    loop {
        if cursor == 0 || cursor == (blocks - 1) * BLOCK_SIZE {
            return cursor + BLOCK_SIZE;
        }
        index.seek(SeekFrom::Start(cursor)).unwrap();
        index.read_exact(&mut word).unwrap();
        index.seek(SeekFrom::Start(cursor + BLOCK_SIZE)).unwrap();
        index.read_exact(&mut next).unwrap();

        for (i, &c) in head.iter().enumerate().rev() {
            if c != 0 {
                break;
            }
            (word[i], next[i]) = (0, 0);
        }

        debug!("{:?}", String::from_utf8(word.iter().map(|&v| v).collect()));
        if word < *head && *head <= next {
            return cursor + BLOCK_SIZE;
        }
        if *head <= word {
            to = cursor;
            cursor -= (cursor - fr) / BLOCK_SIZE / 2 * BLOCK_SIZE;
        } else if word < *head {
            fr = cursor;
            cursor += (to - cursor) / BLOCK_SIZE / 2 * BLOCK_SIZE;
        }
    }
}
