use uuid::Uuid;

use crate::domain::search::{
    SearchQuery, SearchResultItem, SearchResultKind, SearchResultPage, SearchSort,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchPostRow {
    pub post_id: Uuid,
    pub title: String,
    pub summary: String,
    pub author_name: String,
    pub category_name: Option<String>,
    pub tags: Vec<String>,
    pub score: i64,
    pub total_count: i64,
}

impl SearchPostRow {
    fn into_item(self, query: &str) -> SearchResultItem {
        SearchResultItem {
            id: self.post_id.to_string(),
            kind: SearchResultKind::Post,
            title_highlighted: highlight(&self.title, query),
            summary_highlighted: highlight(&self.summary, query),
            title: self.title,
            summary: self.summary,
            category_name: self.category_name,
            tags: self.tags,
            author_name: self.author_name,
            score: self.score,
            url: format!("/posts/{}", self.post_id),
        }
    }
}

pub struct PostgresSearchRepository;

impl PostgresSearchRepository {
    pub async fn search_posts(
        pool: &sqlx::PgPool,
        query: SearchQuery,
    ) -> sqlx::Result<SearchResultPage> {
        let query = query.normalized();
        let pattern = format!("%{}%", query.q);
        let category = query.category.as_deref();
        let tag = query.tag.as_deref();
        let sort = match query.sort {
            SearchSort::Relevance => "relevance",
            SearchSort::Latest => "latest",
            SearchSort::Hot => "hot",
        };
        let limit = query.page_size as i64;
        let offset = ((query.page - 1) * query.page_size) as i64;
        let rows = sqlx::query_as!(
            SearchPostRow,
            r#"
with matched as (
    select
        p.post_id,
        p.title,
        p.summary,
        u.nickname as author_name,
        c.name as category_name,
        coalesce(array_remove(array_agg(t.name order by t.name), null), array[]::text[]) as tags,
        (
            p.view_count
            + p.comment_count * 10
            + p.like_count * 5
            + p.favorite_count * 5
            + case when p.title ilike $1 then 100 else 0 end
            + case when p.summary ilike $1 then 40 else 0 end
        ) as score,
        p.published_at
    from posts p
    join users u on u.user_id = p.author_id
    left join categories c on c.category_id = p.category_id
    left join post_contents pc on pc.post_id = p.post_id
    left join post_tags pt on pt.post_id = p.post_id
    left join tags t on t.tag_id = pt.tag_id
    where p.status = 'published'
      and ($2 = '' or (
          p.title ilike $1
          or p.summary ilike $1
          or pc.markdown ilike $1
          or c.name ilike $1
          or exists (
              select 1
              from post_tags qpt
              join tags qt on qt.tag_id = qpt.tag_id
              where qpt.post_id = p.post_id
                and qt.name ilike $1
          )
      ))
      and ($3::text is null or c.name = $3)
      and ($4::text is null or exists (
          select 1
          from post_tags fpt
          join tags ft on ft.tag_id = fpt.tag_id
          where fpt.post_id = p.post_id
            and ft.name = $4
      ))
    group by
        p.post_id,
        p.title,
        p.summary,
        u.nickname,
        c.name,
        p.view_count,
        p.comment_count,
        p.like_count,
        p.favorite_count,
        p.published_at
)
select
    post_id,
    title,
    summary,
    author_name,
    category_name as "category_name?",
    tags as "tags!: Vec<String>",
    score as "score!",
    count(*) over() as "total_count!"
from matched
order by
    case when $5 = 'hot' then score else 0 end desc,
    case when $5 = 'latest' then published_at else null end desc,
    score desc,
    post_id desc
limit $6
offset $7
"#,
            pattern,
            query.q,
            category,
            tag,
            sort,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        let total = rows.first().map(|row| row.total_count).unwrap_or(0).max(0) as usize;
        let items = rows
            .into_iter()
            .map(|row| row.into_item(&query.q))
            .collect();

        Ok(SearchResultPage {
            query,
            total,
            items,
            suggestions: vec![
                "leptos server functions".to_string(),
                "sqlx transaction".to_string(),
                "axum middleware".to_string(),
            ],
        })
    }
}

fn highlight(value: &str, query: &str) -> String {
    let query = query.trim();
    if query.is_empty() {
        return value.to_string();
    }

    let lower_value = value.to_lowercase();
    let lower_query = query.to_lowercase();
    if let Some(start) = lower_value.find(&lower_query) {
        let end = start + lower_query.len();
        format!(
            "{}<mark>{}</mark>{}",
            &value[..start],
            &value[start..end],
            &value[end..]
        )
    } else {
        value.to_string()
    }
}
