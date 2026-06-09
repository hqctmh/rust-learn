-- Add migration script here
create table if not exists config
(
    id          integer primary key autoincrement,
    name        text              not null,
    value       text              not null,
    state       integer           not null,
    create_time text    default CURRENT_TIMESTAMP,
    update_time text    default CURRENT_TIMESTAMP,
    version     integer default 0 not null
);

create index idx_name on config (name);