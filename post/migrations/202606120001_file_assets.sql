create table if not exists file_assets (
    file_id uuid primary key,
    original_filename text not null,
    bucket text not null,
    storage_key text not null unique,
    file_size bigint not null check (file_size > 0),
    mime_type text not null,
    file_hash text not null unique,
    usage text not null check (usage in ('avatar', 'cover_image', 'markdown_image', 'announcement_image')),
    uploader_id uuid not null references users(user_id) on delete restrict,
    public_url text not null,
    markdown_image text not null,
    uploaded_at timestamptz not null default now()
);

create index if not exists file_assets_uploader_uploaded_idx on file_assets(uploader_id, uploaded_at desc);
