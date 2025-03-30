-- Add migration script here
create table task(
    id integer not null primary key autoincrement,
    deadline date,
    content text
);
