/*!
    Provides functionality to manage kondo tasks
    in the database.
!*/
use chrono::NaiveDate;
use sqlx::{Error, Row, SqlitePool};

use crate::kondo::Task;

/// Inserts a task into the kondo DB and returns the generated Id or
/// error if the insert fails.
pub async fn insert_task(pool: &SqlitePool, task: &Task) -> Result<i64, Error> {
    let insert_stmt = r#"
            insert into task(deadline, content) values($1, $2)
        "#;
    match sqlx::query(insert_stmt)
        .bind(&task.deadline.to_string())
        .bind(&task.content)
        .execute(pool)
        .await
    {
        Ok(res) => Ok(res.last_insert_rowid()),
        Err(e) => Err(e),
    }
}

pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Task>, Error> {
    let select_stmt = r#"
        select * from task
        "#;
    match sqlx::query(select_stmt).fetch_all(pool).await {
        Ok(rows) => {
            let tasks = rows
                .iter()
                .map(|row| {
                    Task::new(
                        NaiveDate::parse_from_str(row.get::<&str, _>("deadline"), "%Y-%m-%d")
                            .expect("Can't parse date from db."),
                        row.get::<&str, _>("content"),
                    )
                })
                .collect();
            Ok(tasks)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{database::list_all, kondo::Task};

    use super::insert_task;

    #[tokio::test]
    async fn test_insert_task() {
        let pool = &sqlx::SqlitePool::connect_lazy("sqlite:kondo-test.db").unwrap();
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .expect("Couldn't run sqlx migrate.");
        let _ = sqlx::query("delete from task").execute(pool).await;
        let _ = sqlx::query("UPDATE sqlite_sequence SET seq = 0 WHERE name = 'task'")
            .execute(pool)
            .await;

        let task = Task::new(
            NaiveDate::from_ymd_opt(2025, 3, 12).unwrap(),
            "Test content",
        );
        let res = insert_task(pool, &task).await;
        assert_eq!(res.unwrap(), 1)
    }

    #[tokio::test]
    async fn test_list_all() {
        let pool = &sqlx::SqlitePool::connect_lazy("sqlite:kondo-test.db").unwrap();
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .expect("Couldn't run sqlx migrate.");
        let _ = sqlx::query("delete from task").execute(pool).await;
        let _ = sqlx::query("UPDATE sqlite_sequence SET seq = 0 WHERE name = 'task'")
            .execute(pool)
            .await;

        let task = Task::new(
            NaiveDate::from_ymd_opt(2025, 3, 12).unwrap(),
            "Test content",
        );
        let _ = insert_task(pool, &task).await;

        let list = list_all(pool).await;
        assert_eq!(list.unwrap().len(), 1);
    }
}
