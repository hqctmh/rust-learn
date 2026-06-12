create table if not exists integration_outbox (
    outbox_id uuid primary key default gen_random_uuid(),
    action_kind text not null check (action_kind in ('cache_invalidate', 'nats_publish', 'search_index')),
    subject text not null,
    aggregate_id uuid,
    payload text not null,
    status text not null default 'pending' check (status in ('pending', 'processed', 'failed')),
    attempts integer not null default 0 check (attempts >= 0),
    last_error text,
    created_at timestamptz not null default now(),
    processed_at timestamptz
);

create index if not exists integration_outbox_pending_idx
    on integration_outbox (status, created_at)
    where status = 'pending';
