alter table posts
add column if not exists is_locked boolean not null default false;
