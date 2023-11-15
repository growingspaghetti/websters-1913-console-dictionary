use encoding_rs;
use std::fs;
use std::io::BufWriter;
use std::io::Write;

use crate::eijiro_text_appender;

pub fn _setup_tanaka_examples() {
    const TANAKA: &str = "eiji-dict/tanaka-examples.utf";
    println!("building the index of {}", TANAKA);
    let utf8 = fs::read_to_string(TANAKA).unwrap();
    let utf8 = utf8
        .lines()
        .filter(|s| s.starts_with("A: "))
        .map(|s| &s[3..])
        .map(|s| s.rsplitn(2, '#').collect::<Vec<&str>>()[1])
        .collect::<Vec<&str>>()
        .join("\n");
    let mut words: Vec<[u8; 20]> = Vec::with_capacity(utf8.len());
    let mut acc = 0u32;
    for line in utf8.lines().map(|v| v.to_string()) {
        add_segments(&mut words, &line, acc);
        acc += line.len() as u32;
        acc += "\n".len() as u32;
    }
    words.sort();
    words.dedup();
    write_indices(words, super::TANAKA_NGRAM, super::TANAKA_INDEX);
    fs::write(
        super::TANAKA_TEXT,
        &utf8.lines().collect::<Vec<&str>>().join("\n"),
    )
    .unwrap();
    println!("indexing finished successfully.");
}

pub fn _setup_ted() {
    const EN: &str = "eiji-dict/ted_train_en-ja.raw.en";
    const JA: &str = "eiji-dict/ted_train_en-ja.raw.ja";
    println!("building the index of {} and {}", EN, JA);
    let en = fs::read_to_string(EN).unwrap();
    let ja = fs::read_to_string(JA).unwrap();
    let utf8 = en
        .lines()
        .zip(ja.lines())
        .map(|(x, y)| [x, y].join("\t"))
        .collect::<Vec<String>>()
        .join("\n");
    let mut words: Vec<[u8; 20]> = Vec::with_capacity(utf8.len());
    let mut acc = 0u32;
    for line in utf8.lines().map(|v| v.to_string()) {
        add_segments(&mut words, &line, acc);
        acc += line.len() as u32;
        acc += "\n".len() as u32;
    }
    words.sort();
    words.dedup();
    write_indices(words, super::TED_NGRAM, super::TED_INDEX);
    fs::write(
        super::TED_TEXT,
        &utf8.lines().collect::<Vec<&str>>().join("\n"),
    )
    .unwrap();
    println!("indexing finished successfully.");
}

pub fn _setup_subtitle() {
    const SUBTITLE: &str = "eiji-dict/train";
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
    write_indices(words, super::SUBTITLE_NGRAM, super::SUBTITLE_INDEX);
    fs::write(
        super::SUBTITLE_TEXT,
        &utf8.lines().collect::<Vec<&str>>().join("\n"),
    )
    .unwrap();
    println!("indexing finished successfully.");
}

pub fn _setup_edict() {
    const EDICT: &str = "eiji-dict/edict.tab";
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
    write_indices(words, super::EDICT_NGRAM, super::EDICT_INDEX);
    fs::write(
        super::EDICT_TEXT,
        &utf8.lines().collect::<Vec<&str>>().join("\n"),
    )
    .unwrap();
    println!("indexing finished successfully.");
}

pub fn setup_eijiro() {
    println!("building the index of {}", super::EIJIRO);
    let sjis = fs::read(super::EIJIRO).unwrap();
    let (utf8, _, _) = encoding_rs::SHIFT_JIS.decode(&sjis);
    let mut buf = eijiro_text_appender::TextAppender::new(utf8.len());
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
    write_indices(words, super::EIJIRO_NGRAM, super::EIJIRO_INDEX);
    fs::write(super::EIJIRO_TEXT, &buf.text[1..]).unwrap();
    println!("indexing finished successfully.");
}

pub fn setup_reijiro() {
    println!("building the index of {}", super::REIJIRO);
    let sjis = fs::read(super::REIJIRO).unwrap();
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
    write_indices(words, super::REIJIRO_NGRAM, super::REIJIRO_INDEX);
    fs::write(super::REIJIRO_TEXT, &text[1..]).unwrap();
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
