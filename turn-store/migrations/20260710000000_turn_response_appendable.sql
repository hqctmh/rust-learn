alter table turn_response
    add column if not exists appendable boolean not null default false;

comment on column turn_response.appendable is '是否为同一 turn、同一 type 合并追加的回复';

create unique index if not exists uq_turn_response_appendable_type
    on turn_response (turn_id, type)
    where appendable;
