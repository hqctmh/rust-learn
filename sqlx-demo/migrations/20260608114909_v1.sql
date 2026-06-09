-- Add migration script here
create table if not exists people
(
    id     bigserial primary key,
    person jsonb not null
)