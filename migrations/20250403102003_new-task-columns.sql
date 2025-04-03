-- Add migration script here
alter table task add category text null;
alter table task add done int not null default 0;
