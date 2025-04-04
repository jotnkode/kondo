use chrono::{Days, NaiveDate, Utc};
use clap::{Parser, Subcommand};
use config::Configuration;
use database::insert_task;
use kondo::Task;
use list_ui::{open_task_editor, run};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use thiserror::Error;
use tokio::sync::OnceCell;

mod config;
mod content_parser;
mod database;
mod kondo;
mod list_ui;

const DB_FILE: &str = "kondo.db";
static CFG: OnceCell<Configuration> = OnceCell::const_new();

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


async fn edit(_file_name: &str) {
    todo!()
}

async fn add(pool: &SqlitePool, date: &Option<NaiveDate>, content: &Option<String>) {
    let cfg = CFG.get().expect("Can't load configuration");
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
        let mut task = Task::new(None, deadline, "");
        task = open_task_editor(task)
            .await
            .expect("Can't get task from editor");
        _ = insert_task(pool, &task).await;
    } else {
        if let (Some(deadline), Some(content)) = (date.clone(), content.clone()) {
            let task = Task::new(None, deadline, &content);
            _ = insert_task(pool, &task).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let cfg = crate::config::Configuration::new();
    _ = CFG.set(cfg);

    let options = SqliteConnectOptions::new()
        .filename(DB_FILE)
        .create_if_missing(true);

    let pool = sqlx::SqlitePool::connect_lazy_with(options);
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

            run(&pool).await.unwrap()
        },
    }
}
