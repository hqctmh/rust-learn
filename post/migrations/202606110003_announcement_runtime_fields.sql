alter table announcements
add column if not exists audience_type text not null default 'all_users'
    check (audience_type in ('all_users', 'user_ids')),
add column if not exists audience_user_ids uuid[] not null default '{}',
add column if not exists published_at timestamptz,
add column if not exists withdrawn_at timestamptz;
