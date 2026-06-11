use uuid::Uuid;

use crate::domain::taxonomy::{CategoryItem, TagItem};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CategoryRow {
    pub category_id: Uuid,
    pub name: String,
    pub color: String,
    pub sort_order: i32,
    pub enabled: bool,
    pub post_count: i64,
}

impl From<CategoryRow> for CategoryItem {
    fn from(row: CategoryRow) -> Self {
        Self {
            category_id: row.category_id,
            name: row.name,
            color: row.color,
            sort_order: row.sort_order,
            enabled: row.enabled,
            post_count: row.post_count.max(0) as u32,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagRow {
    pub tag_id: Uuid,
    pub name: String,
    pub sort_order: i32,
    pub enabled: bool,
    pub use_count: i64,
}

impl From<TagRow> for TagItem {
    fn from(row: TagRow) -> Self {
        Self {
            tag_id: row.tag_id,
            name: row.name,
            sort_order: row.sort_order,
            enabled: row.enabled,
            use_count: row.use_count.max(0) as u32,
        }
    }
}

pub struct PostgresTaxonomyRepository;

impl PostgresTaxonomyRepository {
    pub fn list_categories_sql() -> &'static str {
        r#"
select
    c.category_id,
    c.name,
    c.color,
    c.sort_order,
    c.enabled,
    count(p.post_id) as post_count
from categories c
left join posts p on p.category_id = c.category_id and p.status = 'published'
group by c.category_id, c.name, c.color, c.sort_order, c.enabled
order by c.sort_order asc, c.name asc
"#
    }

    pub fn list_tags_sql() -> &'static str {
        r#"
select
    t.tag_id,
    t.name,
    t.sort_order,
    t.enabled,
    greatest(t.use_count, count(p.post_id)) as use_count
from tags t
left join post_tags pt on pt.tag_id = t.tag_id
left join posts p on p.post_id = pt.post_id and p.status = 'published'
group by t.tag_id, t.name, t.sort_order, t.enabled, t.use_count
order by greatest(t.use_count, count(p.post_id)) desc, t.sort_order asc, t.name asc
"#
    }

    pub async fn public_categories(pool: &sqlx::PgPool) -> sqlx::Result<Vec<CategoryItem>> {
        list_categories(pool, true).await
    }

    pub async fn admin_categories(pool: &sqlx::PgPool) -> sqlx::Result<Vec<CategoryItem>> {
        list_categories(pool, false).await
    }

    pub async fn find_category(
        pool: &sqlx::PgPool,
        category_id: Uuid,
    ) -> sqlx::Result<Option<CategoryItem>> {
        let row = sqlx::query_as!(
            CategoryRow,
            r#"
select
    c.category_id,
    c.name,
    c.color,
    c.sort_order,
    c.enabled,
    count(p.post_id) as "post_count!"
from categories c
left join posts p on p.category_id = c.category_id and p.status = 'published'
where c.category_id = $1
group by c.category_id, c.name, c.color, c.sort_order, c.enabled
limit 1
"#,
            category_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(CategoryItem::from))
    }

    pub async fn enabled_category_name_exists(
        pool: &sqlx::PgPool,
        name: &str,
        ignore_id: Option<Uuid>,
    ) -> sqlx::Result<bool> {
        let row = sqlx::query!(
            r#"
select exists (
    select 1
    from categories
    where enabled = true
      and name = $1
      and ($2::uuid is null or category_id <> $2)
) as "exists!"
"#,
            name.trim(),
            ignore_id
        )
        .fetch_one(pool)
        .await?;

        Ok(row.exists)
    }

    pub async fn insert_category(
        pool: &sqlx::PgPool,
        category: &CategoryItem,
    ) -> sqlx::Result<CategoryItem> {
        let slug = stable_slug("category", &category.name, category.category_id);
        let row = sqlx::query_as!(
            CategoryRow,
            r#"
insert into categories (
    category_id,
    name,
    slug,
    color,
    sort_order,
    enabled
)
values ($1, $2, $3, $4, $5, $6)
returning
    category_id,
    name,
    color,
    sort_order,
    enabled,
    0::bigint as "post_count!"
"#,
            category.category_id,
            category.name,
            slug,
            category.color,
            category.sort_order,
            category.enabled
        )
        .fetch_one(pool)
        .await?;

        Ok(CategoryItem::from(row))
    }

    pub async fn update_category(
        pool: &sqlx::PgPool,
        category: &CategoryItem,
    ) -> sqlx::Result<Option<CategoryItem>> {
        let row = sqlx::query_as!(
            CategoryRow,
            r#"
update categories
set
    name = $2,
    color = $3,
    sort_order = $4,
    enabled = $5
where category_id = $1
returning
    category_id,
    name,
    color,
    sort_order,
    enabled,
    (
        select count(p.post_id)
        from posts p
        where p.category_id = categories.category_id
          and p.status = 'published'
    ) as "post_count!"
"#,
            category.category_id,
            category.name,
            category.color,
            category.sort_order,
            category.enabled
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(CategoryItem::from))
    }

    pub async fn public_tags(pool: &sqlx::PgPool) -> sqlx::Result<Vec<TagItem>> {
        list_tags(pool, true).await
    }

    pub async fn admin_tags(pool: &sqlx::PgPool) -> sqlx::Result<Vec<TagItem>> {
        list_tags(pool, false).await
    }

    pub async fn find_tag(pool: &sqlx::PgPool, tag_id: Uuid) -> sqlx::Result<Option<TagItem>> {
        let row = sqlx::query_as!(
            TagRow,
            r#"
select
    t.tag_id,
    t.name,
    t.sort_order,
    t.enabled,
    greatest(t.use_count, count(p.post_id)) as "use_count!"
from tags t
left join post_tags pt on pt.tag_id = t.tag_id
left join posts p on p.post_id = pt.post_id and p.status = 'published'
where t.tag_id = $1
group by t.tag_id, t.name, t.sort_order, t.enabled, t.use_count
limit 1
"#,
            tag_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(TagItem::from))
    }

    pub async fn enabled_tag_name_exists(
        pool: &sqlx::PgPool,
        name: &str,
        ignore_id: Option<Uuid>,
    ) -> sqlx::Result<bool> {
        let normalized = name.trim().to_lowercase();
        let row = sqlx::query!(
            r#"
select exists (
    select 1
    from tags
    where enabled = true
      and name = $1
      and ($2::uuid is null or tag_id <> $2)
) as "exists!"
"#,
            normalized,
            ignore_id
        )
        .fetch_one(pool)
        .await?;

        Ok(row.exists)
    }

    pub async fn insert_tag(pool: &sqlx::PgPool, tag: &TagItem) -> sqlx::Result<TagItem> {
        let slug = stable_slug("tag", &tag.name, tag.tag_id);
        let row = sqlx::query_as!(
            TagRow,
            r#"
insert into tags (
    tag_id,
    name,
    slug,
    sort_order,
    enabled,
    use_count
)
values ($1, $2, $3, $4, $5, $6)
returning
    tag_id,
    name,
    sort_order,
    enabled,
    use_count
"#,
            tag.tag_id,
            tag.name,
            slug,
            tag.sort_order,
            tag.enabled,
            tag.use_count as i64
        )
        .fetch_one(pool)
        .await?;

        Ok(TagItem::from(row))
    }

    pub async fn update_tag(pool: &sqlx::PgPool, tag: &TagItem) -> sqlx::Result<Option<TagItem>> {
        let row = sqlx::query_as!(
            TagRow,
            r#"
update tags
set
    name = $2,
    sort_order = $3,
    enabled = $4,
    use_count = $5
where tag_id = $1
returning
    tag_id,
    name,
    sort_order,
    enabled,
    use_count
"#,
            tag.tag_id,
            tag.name,
            tag.sort_order,
            tag.enabled,
            tag.use_count as i64
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(TagItem::from))
    }

    pub async fn merge_tag(
        pool: &sqlx::PgPool,
        source: &TagItem,
        target: &TagItem,
    ) -> sqlx::Result<Option<TagItem>> {
        let mut tx = pool.begin().await?;

        sqlx::query!(
            r#"
insert into post_tags (post_id, tag_id)
select post_id, $2
from post_tags
where tag_id = $1
on conflict do nothing
"#,
            source.tag_id,
            target.tag_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
delete from post_tags
where tag_id = $1
"#,
            source.tag_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
update tags
set enabled = false,
    use_count = 0
where tag_id = $1
"#,
            source.tag_id
        )
        .execute(&mut *tx)
        .await?;

        let next_use_count = target.use_count.saturating_add(source.use_count) as i64;
        let row = sqlx::query_as!(
            TagRow,
            r#"
update tags
set enabled = true,
    use_count = $2
where tag_id = $1
returning
    tag_id,
    name,
    sort_order,
    enabled,
    use_count
"#,
            target.tag_id,
            next_use_count
        )
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(row.map(TagItem::from))
    }
}

async fn list_categories(
    pool: &sqlx::PgPool,
    only_enabled: bool,
) -> sqlx::Result<Vec<CategoryItem>> {
    let rows = sqlx::query_as!(
        CategoryRow,
        r#"
select
    c.category_id,
    c.name,
    c.color,
    c.sort_order,
    c.enabled,
    count(p.post_id) as "post_count!"
from categories c
left join posts p on p.category_id = c.category_id and p.status = 'published'
where ($1 = false or c.enabled = true)
group by c.category_id, c.name, c.color, c.sort_order, c.enabled
order by c.sort_order asc, c.name asc
"#,
        only_enabled
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(CategoryItem::from).collect())
}

async fn list_tags(pool: &sqlx::PgPool, only_enabled: bool) -> sqlx::Result<Vec<TagItem>> {
    let rows = sqlx::query_as!(
        TagRow,
        r#"
select
    t.tag_id,
    t.name,
    t.sort_order,
    t.enabled,
    greatest(t.use_count, count(p.post_id)) as "use_count!"
from tags t
left join post_tags pt on pt.tag_id = t.tag_id
left join posts p on p.post_id = pt.post_id and p.status = 'published'
where ($1 = false or t.enabled = true)
group by t.tag_id, t.name, t.sort_order, t.enabled, t.use_count
order by greatest(t.use_count, count(p.post_id)) desc, t.sort_order asc, t.name asc
"#,
        only_enabled
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(TagItem::from).collect())
}

fn stable_slug(prefix: &str, name: &str, id: Uuid) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in name.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        format!("{prefix}-{id}")
    } else {
        slug
    }
}
