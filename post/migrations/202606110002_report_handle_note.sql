alter table reports
add column if not exists handle_note text not null default '';
