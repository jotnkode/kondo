use std::{
    io::{Read, Write},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
};

use chrono::{Days, NaiveDate, Utc};
use clap::{Parser, Subcommand};
use content_parser::parse_task;
use database::insert_task;
use kondo::Task;
use list_ui::run;
use sqlx::SqlitePool;
use tempfile::{NamedTempFile, tempfile};
use thiserror::Error;
use tokio::fs;
use tokio::process::{self, Command};

mod config;
mod content_parser;
mod database;
mod kondo;
mod list_ui;

#[derive(Debug, Error)]
enum Error {}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cmd {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        #[arg(name = "date", short, long, value_name = "DATE")]
        date: Option<NaiveDate>,
        #[arg(name = "content", short, long, value_name = "DATE")]
        content: Option<String>,
    },
    Edit {
        #[arg(short, long, value_name = "FILE")]
        file_name: String,
    },
    List {},
}

async fn open_task_editor(task: Task) -> Result<Task, Error> {
    let content = format!(
        "[{}]\n{}",
        task.deadline.format("%Y-%m-%d").to_string(),
        task.content
    );
    let mut tmpf = NamedTempFile::new().expect("Can't create temp file");
    let _ = tmpf.write_all(content.as_bytes());
    let file_name = tmpf.into_temp_path();

    let mut cmd = Command::new("vim");
    cmd.arg(
        file_name
            .as_os_str()
            .to_str()
            .expect("Can't convert path to string"),
    );
    let mut vim = cmd.spawn().expect("Can't open editor.");
    let _ = vim.wait().await;
    let updated_content = fs::read_to_string(file_name.as_os_str().to_str().unwrap())
        .await
        .expect("Couldn't read file.");

    let new_task = parse_task(&mut updated_content.as_str()).expect("Couldn't parse task");
    Ok(new_task)
}

async fn edit(_file_name: &str) {
    todo!()
}

async fn add(pool: &SqlitePool, date: &Option<NaiveDate>, content: &Option<String>) {
    let cfg = crate::config::Configuration::new();
    let days = Days::new(
        cfg.kondo
            .default_deadline
            .parse::<u64>()
            .expect("default_deadline is non-numeric"),
    );
    if *date == None && *content == None {
        let deadline = Utc::now()
            .naive_utc()
            .date()
            .checked_add_days(days)
            .expect("Invalid deadline date.");
        let mut task = Task::new(deadline, "");
        task = open_task_editor(task)
            .await
            .expect("Can't get task from editor");
        _ = insert_task(pool, &task).await;
    } else {
        if let (Some(_d), Some(_c)) = (date.clone(), content.clone()) {}
    }
}

#[tokio::main]
async fn main() {
    let pool = sqlx::SqlitePool::connect_lazy("sqlite:kondo-test.db").unwrap();
    match sqlx::migrate!().run(&pool).await {
        Ok(_) => println!("Database setup complete."),
        Err(e) => eprintln!("Couldn't complete database setup.\n{}", e.to_string()),
    }

    let cmds = Cmd::parse();
    match &cmds.commands {
        Commands::Edit { file_name } => {
            edit(file_name).await;
        }
        Commands::Add { date, content } => add(&pool, date, content).await,
        Commands::List {} =>  {
            let pool = sqlx::SqlitePool::connect_lazy("sqlite:kondo-test.db").unwrap();
            run(&pool).await.unwrap()
        },
    }
}
