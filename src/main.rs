use encoding_rs;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rayon::prelude::*;
use sqlx::sqlite::SqlitePool;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

const SUBTITLE_NGRAM: &str = "SUBTITLE_NGRAM";
const SUBTITLE_INDEX: &str = "SUBTITLE_INDEX";
const SUBTITLE_DB: &str = "subtitles.db";
const EIJIRO: &str = "EIJIRO-1448.TXT";
const EIJIROGZIP: &str = "EIJIRO-1448.tsv.gz";
const EIJIRO_NGRAM: &str = "EIJIRO-1448_NGRAM";
const EIJIRO_INDEX: &str = "EIJIRO-1448_INDEX";
const EIJIRO_DB: &str = "eijiro.db";
const REIJIRO: &str = "REIJI-1441.TXT";
const REIJIROGZIP: &str = "REIJI-1441.tsv.gz";
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

fn setup_reijiro() {
    println!("converting the format of {}", REIJIRO);
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
    let gzip = std::fs::File::create(REIJIROGZIP)
        .expect(format!("ERROR unable to create {}", REIJIROGZIP).as_str());
    let mut w = GzEncoder::new(gzip, Compression::default());
    w.write_all(text[1..].as_bytes())
        .expect(format!("ERROR failed to write data into {}", REIJIROGZIP).as_str());
    w.flush().unwrap();
    println!("conversion finished successfully.");
}

fn load_reijiro<'a>(text: &'a mut String) -> Vec<&'a str> {
    let gzip_path = Path::new(REIJIROGZIP);
    let source_path = Path::new(REIJIRO);
    println!("{} exists:{}", REIJIROGZIP, gzip_path.exists());
    println!("{} exists:{}", REIJIRO, source_path.exists());
    if source_path.exists() && !gzip_path.exists() {
        setup_reijiro();
    }
    if !gzip_path.exists() {
        return vec![];
    }
    let r = std::fs::File::open(gzip_path).unwrap();
    let mut decoder = GzDecoder::new(r);
    decoder.read_to_string(text).unwrap();
    text.lines().collect()
}

fn load_edict_eijiro<'a>(edict_text: &'a mut String) -> Vec<&'a str> {
    let gzip_path = Path::new(EIJIROGZIP);
    let source_path = Path::new(EIJIRO);
    println!("{} exists:{}", EIJIROGZIP, gzip_path.exists());
    println!("{} exists:{}", EIJIRO, source_path.exists());
    if source_path.exists() && !gzip_path.exists() {
        setup_eijiro();
    }
    if !gzip_path.exists() {
        return edict_text.lines().collect();
    }
    let r = std::fs::File::open(gzip_path).unwrap();
    let mut decoder = GzDecoder::new(r);
    decoder.read_to_string(edict_text).unwrap();
    edict_text.lines().collect()
}

fn reorder<'a>(hits: &Vec<&'a str>, input: String) -> Vec<&'a str> {
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
    // let mut edict_text = String::new(); //EDICT.to_string();
    // let dicts = load_edict_eijiro(&mut edict_text);
    // let mut reiji = String::new();
    // let reijiro = load_reijiro(&mut reiji);

    println!("\x1b[0m\x1b[1;32m検索文字\x1b[0m(Enter)で検索");
    println!("\x1b[1;33md\x1b[0mで画面をスクロール \x1b[1;33mq\x1b[0mで次の辞書");
    println!("\x1b[1;36mctrl+c\x1b[0mでソフトウェアを終了");

    loop {
        let input: String = get_input("");
        if input.trim().is_empty() {
            continue;
        }

        // {
        //     let nums = ngram_search(&input, EIJIRO_NGRAM, EIJIRO_INDEX);
        //     print_results(filter(&input, &nums));
        // }
        {
            let nums = ngram_search(&input, SUBTITLE_NGRAM, SUBTITLE_INDEX);
            print_results(filter(&input, &nums, SUBTITLE_DB).await.unwrap());
        }
    }
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
    let high_light_left = format!(
        "\x1b[0m\x1b[1;32m{}\x1b[0m\x1b[1;36m",
        input.replace("\t", "")
    );
    let high_light_right = format!("\x1b[1;32m{}\x1b[0m", input);

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
    //let conn = pool.acquire().await?;
    let rows = query.fetch_all(&pool).await?;
    // for r in rows {
    //     println!("{:?}", r.line);
    // }

    let hits = &rows
        .par_iter()
        .map(|r| r.line.as_str())
        .filter(|l| l.contains(input))
        .collect::<Vec<&str>>();

    let results = reorder(&hits, input.to_string())
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
        .collect::<Vec<String>>();
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
    nums
}

/// 8 byte length segmentation
const BLOCK_SIZE: u64 = 8;

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

        println!("{:?}", String::from_utf8(word.iter().map(|&v| v).collect()));
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

        println!("{:?}", String::from_utf8(word.iter().map(|&v| v).collect()));
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
