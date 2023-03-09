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

const EDICT: &str = "eiji-dict/edict.tab";
const EDICT_NGRAM: &str = "EDICT_NGRAM";
const EDICT_INDEX: &str = "EDICT_INDEX";
const EDICT_DB: &str = "edict.db";

const SUBTITLE: &str = "eiji-dict/train";
const SUBTITLE_NGRAM: &str = "SUBTITLE_NGRAM";
const SUBTITLE_INDEX: &str = "SUBTITLE_INDEX";
const SUBTITLE_DB: &str = "subtitles.db";

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    //create_sqlite3(&EDICT_DB, &EDICT).await.unwrap();
    create_sqlite3(&SUBTITLE_DB, &SUBTITLE).await.unwrap();
    Ok(())
}

async fn create_sqlite3(db :&str, text_file: &str) -> Result<(), sqlx::Error> {
    println!("building db of {}", db);
    let utf8 = fs::read_to_string(text_file).unwrap();
    let pool = SqlitePool::connect(format!("sqlite:{}?mode=rwc", db).as_str()).await?;
    let mut conn = pool.acquire().await?;
    sqlx::query("CREATE TABLE lines(id INTEGER PRIMARY KEY AUTOINCREMENT, line TEXT);")
        .execute(&mut conn)
        .await?;
    let mut tx = pool.begin().await?;
    for (i, line) in utf8.lines().map(|v| v.to_string()).enumerate() {
        let query_str = format!("INSERT INTO lines(id, line) VALUES ({},?);", i);
        let mut query = sqlx::query(&query_str);
        query = query.bind(&line);
        tx.execute(query).await?;
    }
    tx.commit().await?;
    println!("db finished successfully.");
    Ok(())
}
