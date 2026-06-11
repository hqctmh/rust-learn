alter table categories
add column if not exists color text not null default '#2563eb',
add column if not exists enabled boolean not null default true;

alter table tags
add column if not exists sort_order integer not null default 0,
add column if not exists enabled boolean not null default true,
add column if not exists use_count bigint not null default 0 check (use_count >= 0);

update categories
set color = case name
    when '公告' then '#2563eb'
    when '教程' then '#16a34a'
    when '问题' then '#f97316'
    when '经验分享' then '#60a5fa'
    when '站务' then '#a855f7'
    else color
end
where color = '#2563eb';
