mod eijiro_text_appender;
mod indexing;
mod print;
mod search;

#[macro_use]
extern crate log;

use std::io;
use std::io::Write;
use std::path::Path;

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
        indexing::setup_reijiro();
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
        indexing::setup_eijiro();
    }
}

fn main() {
    env_logger::init();
    // indexing::_setup_edict();
    // indexing::_setup_subtitle();
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
            let nums = search::ngram_search(&input, SUBTITLE_NGRAM, SUBTITLE_INDEX);
            let hits = search::load_then_filter(&input, &nums, SUBTITLE_TEXT);
            print::print_to_console(&input, hits);
        }
        if Path::new(REIJIRO_TEXT).exists() {
            let nums = search::ngram_search(&input, REIJIRO_NGRAM, REIJIRO_INDEX);
            let hits = search::load_then_filter(&input, &nums, REIJIRO_TEXT);
            print::print_to_console(&input, hits);
        }
    }
}

fn edict_eiji(input: &String) {
    let nums = search::ngram_search(&input, EDICT_NGRAM, EDICT_INDEX);
    let mut hits = search::load_then_filter(&input, &nums, EDICT_TEXT);
    if Path::new(EIJIRO_TEXT).exists() {
        let nums = search::ngram_search(&input, EIJIRO_NGRAM, EIJIRO_INDEX);
        let mut eiji_hits = search::load_then_filter(&input, &nums, EIJIRO_TEXT);
        hits.append(&mut eiji_hits)
    }
    print::print_to_console(input, hits);
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

#[cfg(test)]
mod tests {
    use crate::{
        indexing::_setup_edict,
        search::{load_then_filter, ngram_search},
    };
    use std::str::FromStr;

    #[test]
    fn journey_tests() {
        _setup_edict();
        {
            let keyword = String::from_str("同型").unwrap();
            let occurences = ngram_search(&keyword, super::EDICT_NGRAM, super::EDICT_INDEX);
            assert_eq!(
                occurences,
                vec![(4455295, 102), (7362715, 82), (7364068, 82), (7364580, 78)]
            );
            let findings = load_then_filter(&keyword, &occurences, super::EDICT_TEXT);
            assert_eq!(
                findings,
                vec![
                    "isomorphism\t/aɪsoʊmɔːfɪzəm/aisoùm<ħ>ò</ħ>fizøm/ 同型 [どうけい],同形 [どうけい]",
                    "same pattern\t/sɛɪm pætn/seim pätn/ 同型 [どうけい],同形 [どうけい]",
                    "same shape\t/sɛɪm ʃɛɪp/seim ŝeip/ 同型 [どうけい],同形 [どうけい]",
                    "same type\t/sɛɪm taɪp/seim taip/ 同型 [どうけい],同形 [どうけい]"
                ]
            );
        }
        {
            let keyword = String::from_str("🍁").unwrap();
            let occurences = ngram_search(&keyword, super::EDICT_NGRAM, super::EDICT_INDEX);
            assert_eq!(occurences, vec![]);
            let findings = load_then_filter(&keyword, &occurences, super::EDICT_TEXT);
            assert_eq!(findings, Vec::<String>::new());
        }
        {
            let keyword = String::from_str("!").unwrap();
            let occurences = ngram_search(&keyword, super::EDICT_NGRAM, super::EDICT_INDEX);
            assert_eq!(occurences.len(), 96);
            let findings = load_then_filter(&keyword, &occurences, super::EDICT_TEXT);
            assert_eq!(findings[0], "a happy new year!\t/ə hæpiː njuː -/ø häp<ħ>ï</ħ> nj<ħ>u</ħ> -/ 賀正 [がしょう],賀正 [がせい]");
            assert_eq!(findings[95], "yuck!\t/-/-/ 最低 [さいてい]");
        }
    }
}
