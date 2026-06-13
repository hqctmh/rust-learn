create table if not exists post_reads (
    user_id uuid not null references users(user_id) on delete cascade,
    post_id uuid not null references posts(post_id) on delete cascade,
    read_at timestamptz not null default now(),
    primary key (user_id, post_id)
);

create index if not exists post_reads_post_idx on post_reads(post_id, read_at desc);
