#[macro_use]
extern crate log;

use encoding_rs;
use rayon::prelude::*;
use sqlx::sqlite::SqlitePool;
use sqlx::Executor;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

const EDICT_NGRAM: &str = "EDICT_NGRAM";
const EDICT_INDEX: &str = "EDICT_INDEX";
const EDICT_DB: &str = "edict.db";

const SUBTITLE_NGRAM: &str = "SUBTITLE_NGRAM";
const SUBTITLE_INDEX: &str = "SUBTITLE_INDEX";
const SUBTITLE_DB: &str = "subtitles.db";

const EIJIRO: &str = "EIJIRO-1448.TXT";
const EIJIRO_DB: &str = "eijiro.db";
const EIJIRO_NGRAM: &str = "EIJIRO-1448_NGRAM";
const EIJIRO_INDEX: &str = "EIJIRO-1448_INDEX";

const REIJIRO: &str = "REIJI-1441.TXT";
const REIJIRO_DB: &str = "reiji.db";
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
    fn text_as_bytes(&self) -> &[u8] {
        self.text[1..].as_bytes()
    }
}

async fn setup_eijiro() -> Result<(), sqlx::Error> {
    println!("building the index of {}", EIJIRO);
    let sjis = fs::read(EIJIRO).unwrap();
    let (utf8, _, _) = encoding_rs::SHIFT_JIS.decode(&sjis);
    let mut buf = TextAppender::new(utf8.len());
    for line in utf8.lines() {
        let separator = line.find(" : ").unwrap();
        buf.append(&line[3..separator], &line[separator + 3..]);
    }

    let mut words: Vec<u128> = vec![];
    let pool = SqlitePool::connect(format!("sqlite:{}?mode=rwc", EIJIRO_DB).as_str()).await?;
    let mut conn = pool.acquire().await?;
    sqlx::query("CREATE TABLE lines(id INTEGER PRIMARY KEY AUTOINCREMENT, line TEXT);")
        .execute(&mut conn)
        .await?;
    let mut tx = pool.begin().await?;
    for (i, line) in buf.text_as_bytes().lines().map(|l| l.unwrap()).enumerate() {
        let query_str = format!("INSERT INTO lines(id, line) VALUES ({},?);", i);
        let mut query = sqlx::query(&query_str);
        query = query.bind(&line);
        tx.execute(query).await?;
        add_segments(&mut words, &line, i);
    }
    tx.commit().await?;
    words.sort();
    words.dedup();
    let (gram, idx) = vec_u128_to_u8(&words);
    fs::write(EIJIRO_NGRAM, &gram).unwrap();
    fs::write(EIJIRO_INDEX, &idx).unwrap();
    println!("indexing finished successfully.");
    Ok(())
}

fn add_segments(words: &mut Vec<u128>, line: &String, n: usize) {
    let bs = line.as_bytes();
    for p in 0..bs.len() {
        let mut u = 0u128;
        u |= bs.get(p + 0).map(|&v| v).unwrap_or(0) as u128;
        u <<= 8;
        u |= bs.get(p + 1).map(|&v| v).unwrap_or(0) as u128;
        u <<= 8;
        u |= bs.get(p + 2).map(|&v| v).unwrap_or(0) as u128;
        u <<= 8;
        u |= bs.get(p + 3).map(|&v| v).unwrap_or(0) as u128;

        u <<= 8;
        u |= bs.get(p + 4).map(|&v| v).unwrap_or(0) as u128;
        u <<= 8;
        u |= bs.get(p + 5).map(|&v| v).unwrap_or(0) as u128;
        u <<= 8;
        u |= bs.get(p + 6).map(|&v| v).unwrap_or(0) as u128;
        u <<= 8;
        u |= bs.get(p + 7).map(|&v| v).unwrap_or(0) as u128;

        u <<= 8;
        u |= bs.get(p + 8).map(|&v| v).unwrap_or(0) as u128;
        u <<= 8;
        u |= bs.get(p + 9).map(|&v| v).unwrap_or(0) as u128;
        u <<= 8;
        u |= bs.get(p + 10).map(|&v| v).unwrap_or(0) as u128;
        u <<= 8;
        u |= bs.get(p + 11).map(|&v| v).unwrap_or(0) as u128;

        u <<= 32;
        u |= n as u128;
        words.push(u);
    }
}

pub fn vec_u128_to_u8(data: &Vec<u128>) -> (Vec<u8>, Vec<u8>) {
    let capacity = 32 / 8 * data.len() as usize;
    let mut gram = Vec::<u8>::with_capacity(capacity);
    let mut lnum = Vec::<u8>::with_capacity(capacity);
    for &value in data {
        gram.push((value >> 120) as u8);
        gram.push((value >> 112) as u8);
        gram.push((value >> 104) as u8);
        gram.push((value >> 96) as u8);

        gram.push((value >> 88) as u8);
        gram.push((value >> 80) as u8);
        gram.push((value >> 72) as u8);
        gram.push((value >> 64) as u8);

        gram.push((value >> 56) as u8);
        gram.push((value >> 48) as u8);
        gram.push((value >> 40) as u8);
        gram.push((value >> 32) as u8);

        lnum.push((value >> 24) as u8);
        lnum.push((value >> 16) as u8);
        lnum.push((value >> 8) as u8);
        lnum.push((value >> 0) as u8);
    }
    (gram, lnum)
}

async fn setup_reijiro() -> Result<(), sqlx::Error> {
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

    let mut words: Vec<u128> = vec![];
    let pool = SqlitePool::connect(format!("sqlite:{}?mode=rwc", REIJIRO_DB).as_str()).await?;
    let mut conn = pool.acquire().await?;
    sqlx::query("CREATE TABLE lines(id INTEGER PRIMARY KEY AUTOINCREMENT, line TEXT);")
        .execute(&mut conn)
        .await?;
    let mut tx = pool.begin().await?;
    for (i, line) in text[1..].lines().map(|v| v.to_string()).enumerate() {
        let query_str = format!("INSERT INTO lines(id, line) VALUES ({},?);", i);
        let mut query = sqlx::query(&query_str);
        query = query.bind(&line);
        tx.execute(query).await?;
        add_segments(&mut words, &line, i);
    }
    tx.commit().await?;
    words.sort();
    words.dedup();
    let (gram, idx) = vec_u128_to_u8(&words);
    fs::write(REIJIRO_NGRAM, &gram).unwrap();
    fs::write(REIJIRO_INDEX, &idx).unwrap();
    println!("indexing finished successfully.");
    Ok(())
}

async fn check_reijiro() {
    let source_path = Path::new(REIJIRO);
    let ngram_path = Path::new(REIJIRO_NGRAM);
    let index_path = Path::new(REIJIRO_INDEX);
    let db_path = Path::new(REIJIRO_DB);
    println!("{} exists:{}", REIJIRO, source_path.exists());
    println!("{} exists:{}", REIJIRO_NGRAM, ngram_path.exists());
    println!("{} exists:{}", REIJIRO_INDEX, index_path.exists());
    println!("{} exists:{}", REIJIRO_DB, db_path.exists());
    if source_path.exists() && !index_path.exists() {
        setup_reijiro().await.unwrap();
    }
}

async fn check_eijiro() {
    let source_path = Path::new(EIJIRO);
    let ngram_path = Path::new(EIJIRO_NGRAM);
    let index_path = Path::new(EIJIRO_INDEX);
    let db_path = Path::new(EIJIRO_DB);
    println!("{} exists:{}", EIJIRO, source_path.exists());
    println!("{} exists:{}", EIJIRO_NGRAM, ngram_path.exists());
    println!("{} exists:{}", EIJIRO_INDEX, index_path.exists());
    println!("{} exists:{}", EIJIRO_DB, db_path.exists());
    if source_path.exists() && !index_path.exists() {
        setup_eijiro().await.unwrap();
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

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    env_logger::init();
    check_eijiro().await;
    check_reijiro().await;

    println!("\x1b[0m\x1b[1;32m検索文字\x1b[0m(Enter)で検索");
    println!("\x1b[1;33md\x1b[0mで画面をスクロール \x1b[1;33mq\x1b[0mで次の辞書");
    println!("\x1b[1;36mctrl+c\x1b[0mでソフトウェアを終了");

    loop {
        let input: String = get_input("");
        if input.trim().is_empty() {
            continue;
        }

        edict_eiji(&input).await;
        {
            let nums = ngram_search(&input, SUBTITLE_NGRAM, SUBTITLE_INDEX);
            let hits = filter(&input, &nums, SUBTITLE_DB).await.unwrap();
            print_results(decorate(&input, hits));
        }
        if Path::new(REIJIRO_DB).exists() {
            let nums = ngram_search(&input, REIJIRO_NGRAM, REIJIRO_INDEX);
            let hits = filter(&input, &nums, REIJIRO_DB).await.unwrap();
            print_results(decorate(&input, hits));
        }
    }
}

async fn edict_eiji(input: &String) {
    let nums = ngram_search(&input, EDICT_NGRAM, EDICT_INDEX);
    let mut hits = filter(&input, &nums, EDICT_DB).await.unwrap();
    if Path::new(EIJIRO_DB).exists() {
        let nums = ngram_search(&input, EIJIRO_NGRAM, EIJIRO_INDEX);
        let mut eiji_hits = filter(&input, &nums, EIJIRO_DB).await.unwrap();
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

async fn filter(input: &str, nums: &Vec<u32>, db: &str) -> Result<Vec<String>, sqlx::Error> {
    debug!("{:?} given:{}", nums, nums.len());
    if nums.len() == 0 {
        return Ok(vec![]);
    }

    #[derive(sqlx::FromRow)]
    struct Line {
        line: String,
    }
    let params = format!("?{}", ", ?".repeat(nums.len() - 1));
    let query_str = format!("SELECT line FROM lines WHERE id IN ( { } )", params);
    let mut query = sqlx::query_as::<_, Line>(&query_str);
    for i in nums {
        query = query.bind(i);
    }
    let pool = SqlitePool::connect(format!("sqlite:{}?mode=rwc", db).as_str()).await?;
    let rows = query.fetch_all(&pool).await?;

    let hits = &rows
        .par_iter()
        .map(|r| r.line.as_str())
        .filter(|l| l.contains(input))
        .collect::<Vec<&str>>();

    let results = hits.iter().map(|s| s.to_string()).collect();
    Ok(results)
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

fn ngram_search(keyword: &String, ngram: &str, index: &str) -> Vec<u32> {
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
    let begin = limit_left(&mut ngramf, &search_block) / BLOCK_SIZE * 4;
    let end = limit_right(&mut ngramf, &search_block) / BLOCK_SIZE * 4;
    let mut indexf = File::open(index).unwrap();
    let mut nums = vec![];
    for p in (begin..end).step_by(4) {
        let mut num = [0u8; 4];
        indexf.seek(SeekFrom::Start(p)).unwrap();
        indexf.read_exact(&mut num).unwrap();
        nums.push(to_u32(&num));
    }
    nums.sort();
    nums.dedup();
    if nums.len() > 9999 {
        info!(
            "because it found too much hits of {}, will truncate to 9999",
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
        if cursor == 0 || cursor == (blocks - 1) * BLOCK_SIZE {
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
        if cursor == 0 || cursor == (blocks - 1) * BLOCK_SIZE {
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
