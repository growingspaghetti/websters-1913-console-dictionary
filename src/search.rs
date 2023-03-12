use rayon::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

pub fn ngram_search(keyword: &String, ngram: &str, index: &str) -> Vec<(u32, u32)> {
    if keyword.is_empty() {
        return vec![];
    }
    let mut search_block = [0u8; BLOCK_SIZE as usize];
    for (i, &v) in keyword.as_bytes().iter().enumerate() {
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

pub fn load_then_filter(input: &str, nums: &Vec<(u32, u32)>, text_file: &str) -> Vec<String> {
    let lines = load(nums, text_file);
    lines
        .par_iter()
        .filter(|l| l.contains(input))
        .map(|s| s.to_string())
        .collect()
}

fn load(nums: &Vec<(u32, u32)>, text_file: &str) -> Vec<String> {
    debug!("{:?} given:{}", nums, nums.len());
    if nums.len() == 0 {
        return vec![];
    }
    nums
        .par_iter()
        .map(|&(offset, len)| {
            let mut txtf = File::open(text_file).unwrap();
            txtf.seek(SeekFrom::Start(offset as u64)).unwrap();
            let mut line = vec![0u8; len as usize];
            txtf.read(&mut line[..]).unwrap();
            String::from_utf8(line).unwrap()
        })
        .collect::<Vec<String>>()
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
        index.read(&mut next).unwrap();

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
            let update = (cursor - fr) / BLOCK_SIZE / 2 * BLOCK_SIZE;
            if update == 0 {
                return cursor;
            }
            cursor -= update;
        } else if word <= *head {
            fr = cursor;
            let update = (to - cursor) / BLOCK_SIZE / 2 * BLOCK_SIZE;
            if update == 0 {
                return cursor;
            }
            cursor += update;
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
        index.read(&mut next).unwrap();

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
            let update = (cursor - fr) / BLOCK_SIZE / 2 * BLOCK_SIZE;
            if update == 0 {
                return cursor;
            }
            cursor -= update;
        } else if word < *head {
            fr = cursor;
            let update = (to - cursor) / BLOCK_SIZE / 2 * BLOCK_SIZE;
            if update == 0 {
                return cursor;
            }
            cursor += update;
        }
    }
}
