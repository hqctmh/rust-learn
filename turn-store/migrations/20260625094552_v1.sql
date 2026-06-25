-- Add migration script here
create extension if not exists pgcrypto;

create table if not exists conversation
(
    id          uuid primary key default gen_random_uuid(),
    doc_id      varchar(64)  not null,
    doc_type    varchar(50)  not null,
    user_id     bigint       not null,
    title       varchar(255) not null,
    type        varchar(50)  not null,
    inline_type varchar(50),
    created_at  timestamptz  not null default now(),
    updated_at  timestamptz  not null default now(),
    deleted_at  bigint       not null default 0
);

comment on table conversation is '对话';
comment on column conversation.id is '主键';
comment on column conversation.doc_id is '文档id';
comment on column conversation.doc_type is 'doc，doc_template';
comment on column conversation.user_id is '用户id';
comment on column conversation.title is '对话标题';
comment on column conversation.type is '对话类型，INLINE_EDIT,CHAT_EDIT';
comment on column conversation.inline_type is '行内编辑类型，可为空';
comment on column conversation.created_at is '创建时间';
comment on column conversation.updated_at is '更新时间';
comment on column conversation.deleted_at is '删除时间，0表示未删除';

create index if not exists idx_user_doc_type_deleted_updated
    on conversation (user_id, doc_id, doc_type, type, deleted_at, updated_at);

create index if not exists idx_doc_id
    on conversation (doc_id);

create table if not exists "turn"
(
    id                          uuid primary key default gen_random_uuid(),
    conversation_id             uuid        not null,
    input_context               text        not null,
    document_content_version_id bigint      not null,
    created_at                  timestamptz not null default now(),
    updated_at                  timestamptz not null default now(),
    deleted_at                  bigint      not null default 0
);

comment on table "turn" is '轮次';
comment on column "turn".id is '主键';
comment on column "turn".conversation_id is '对话id';
comment on column "turn".input_context is '用户输入上下文';
comment on column "turn".document_content_version_id is '文档内容版本id';
comment on column "turn".created_at is '创建时间';
comment on column "turn".updated_at is '更新时间';
comment on column "turn".deleted_at is '删除时间，0表示未删除';

create index if not exists idx_conversation_deleted_created
    on "turn" (conversation_id, deleted_at, created_at);

create table if not exists turn_response
(
    id         uuid primary key default gen_random_uuid(),
    turn_id    uuid        not null,
    type       varchar(50) not null,
    response   text        not null,
    created_at timestamptz not null default now()
);

comment on table turn_response is '轮次回复';
comment on column turn_response.id is '主键';
comment on column turn_response.turn_id is '轮次id';
comment on column turn_response.type is '回复类型，如text';
comment on column turn_response.response is '模型回复内容';
comment on column turn_response.created_at is '创建时间';

create index if not exists idx_turn_created
    on turn_response (turn_id, created_at);
