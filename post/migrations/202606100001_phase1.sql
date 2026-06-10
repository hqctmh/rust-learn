create extension if not exists pgcrypto;

create table users (
    user_id uuid primary key default gen_random_uuid(),
    username text not null unique,
    password_hash text not null,
    nickname text not null,
    avatar_url text,
    bio text not null default '',
    status text not null default 'active' check (status in ('active', 'disabled')),
    is_admin boolean not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table sessions (
    session_id uuid primary key default gen_random_uuid(),
    user_id uuid not null references users(user_id) on delete cascade,
    token_hash text not null unique,
    expires_at timestamptz not null,
    created_at timestamptz not null default now()
);

create table roles (
    role_id uuid primary key default gen_random_uuid(),
    code text not null unique,
    name text not null,
    created_at timestamptz not null default now()
);

create table permissions (
    permission_id uuid primary key default gen_random_uuid(),
    code text not null unique,
    name text not null,
    created_at timestamptz not null default now()
);

create table user_roles (
    user_id uuid not null references users(user_id) on delete cascade,
    role_id uuid not null references roles(role_id) on delete cascade,
    created_at timestamptz not null default now(),
    primary key (user_id, role_id)
);

create table role_permissions (
    role_id uuid not null references roles(role_id) on delete cascade,
    permission_id uuid not null references permissions(permission_id) on delete cascade,
    created_at timestamptz not null default now(),
    primary key (role_id, permission_id)
);

create table categories (
    category_id uuid primary key default gen_random_uuid(),
    name text not null unique,
    slug text not null unique,
    sort_order integer not null default 0,
    created_at timestamptz not null default now()
);

create table tags (
    tag_id uuid primary key default gen_random_uuid(),
    name text not null unique,
    slug text not null unique,
    created_at timestamptz not null default now()
);

create table posts (
    post_id uuid primary key default gen_random_uuid(),
    author_id uuid not null references users(user_id) on delete restrict,
    category_id uuid references categories(category_id) on delete set null,
    title text not null,
    summary text not null default '',
    cover_url text,
    status text not null default 'draft' check (status in ('draft', 'published', 'offline', 'deleted')),
    is_recommended boolean not null default false,
    is_pinned boolean not null default false,
    view_count bigint not null default 0 check (view_count >= 0),
    comment_count bigint not null default 0 check (comment_count >= 0),
    like_count bigint not null default 0 check (like_count >= 0),
    favorite_count bigint not null default 0 check (favorite_count >= 0),
    published_at timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table post_contents (
    post_id uuid primary key references posts(post_id) on delete cascade,
    markdown text not null,
    sanitized_html text not null default '',
    updated_at timestamptz not null default now()
);

create table post_tags (
    post_id uuid not null references posts(post_id) on delete cascade,
    tag_id uuid not null references tags(tag_id) on delete cascade,
    primary key (post_id, tag_id)
);

create table comments (
    comment_id uuid primary key default gen_random_uuid(),
    post_id uuid not null references posts(post_id) on delete cascade,
    parent_comment_id uuid references comments(comment_id) on delete cascade,
    author_id uuid not null references users(user_id) on delete restrict,
    content text not null,
    status text not null default 'visible' check (status in ('visible', 'deleted')),
    like_count bigint not null default 0 check (like_count >= 0),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table post_likes (
    post_id uuid not null references posts(post_id) on delete cascade,
    user_id uuid not null references users(user_id) on delete cascade,
    created_at timestamptz not null default now(),
    primary key (post_id, user_id)
);

create table comment_likes (
    comment_id uuid not null references comments(comment_id) on delete cascade,
    user_id uuid not null references users(user_id) on delete cascade,
    created_at timestamptz not null default now(),
    primary key (comment_id, user_id)
);

create table post_favorites (
    post_id uuid not null references posts(post_id) on delete cascade,
    user_id uuid not null references users(user_id) on delete cascade,
    created_at timestamptz not null default now(),
    primary key (post_id, user_id)
);

create table follows (
    follower_id uuid not null references users(user_id) on delete cascade,
    followee_id uuid not null references users(user_id) on delete cascade,
    created_at timestamptz not null default now(),
    primary key (follower_id, followee_id),
    check (follower_id <> followee_id)
);

create table notifications (
    notification_id uuid primary key default gen_random_uuid(),
    recipient_id uuid not null references users(user_id) on delete cascade,
    actor_id uuid references users(user_id) on delete set null,
    notification_type text not null,
    title text not null,
    body text not null,
    target_type text,
    target_id uuid,
    read_at timestamptz,
    created_at timestamptz not null default now()
);

create table announcements (
    announcement_id uuid primary key default gen_random_uuid(),
    title text not null,
    content text not null,
    announcement_type text not null default 'system',
    is_pinned boolean not null default false,
    status text not null default 'draft' check (status in ('draft', 'published', 'offline')),
    starts_at timestamptz,
    ends_at timestamptz,
    creator_id uuid not null references users(user_id) on delete restrict,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table announcement_reads (
    announcement_id uuid not null references announcements(announcement_id) on delete cascade,
    user_id uuid not null references users(user_id) on delete cascade,
    read_at timestamptz not null default now(),
    primary key (announcement_id, user_id)
);

create table files (
    file_id uuid primary key default gen_random_uuid(),
    original_filename text not null,
    bucket text not null,
    object_key text not null,
    file_size bigint not null check (file_size > 0),
    mime_type text not null,
    file_hash text not null,
    uploader_id uuid not null references users(user_id) on delete restrict,
    created_at timestamptz not null default now(),
    unique (bucket, object_key)
);

create table reports (
    report_id uuid primary key default gen_random_uuid(),
    reporter_id uuid not null references users(user_id) on delete restrict,
    target_type text not null check (target_type in ('post', 'comment', 'user')),
    target_id uuid not null,
    reason text not null,
    description text not null default '',
    status text not null default 'pending' check (status in ('pending', 'accepted', 'rejected')),
    handler_id uuid references users(user_id) on delete set null,
    handled_at timestamptz,
    created_at timestamptz not null default now()
);

create table audit_logs (
    audit_log_id uuid primary key default gen_random_uuid(),
    operator_id uuid references users(user_id) on delete set null,
    action text not null,
    target_type text not null,
    target_id uuid,
    before_data jsonb,
    after_data jsonb,
    ip_address text,
    user_agent text,
    created_at timestamptz not null default now()
);

create index posts_published_idx on posts(status, published_at desc);
create index comments_post_created_idx on comments(post_id, created_at);
create index notifications_recipient_created_idx on notifications(recipient_id, created_at desc);
create index audit_logs_created_idx on audit_logs(created_at desc);
