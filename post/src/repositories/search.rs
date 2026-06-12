use elasticsearch::{Elasticsearch, SearchParts, http::transport::Transport};
use serde_json::{Value, json};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchTagRow {
    pub tag_id: Uuid,
    pub name: String,
    pub post_count: i64,
    pub score: i64,
    pub total_count: i64,
}

impl SearchTagRow {
    fn into_item(self, query: &str) -> SearchResultItem {
        let title = format!("#{}", self.name);
        let summary = format!("{} 个主题使用该标签", self.post_count.max(0));
        SearchResultItem {
            id: self.name.clone(),
            kind: SearchResultKind::Tag,
            title_highlighted: highlight(&title, query),
            summary_highlighted: highlight(&summary, query),
            title,
            summary,
            category_name: None,
            tags: vec![self.name.clone()],
            author_name: "标签".to_string(),
            score: self.score,
            url: format!("/search?tag={}", self.name),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchUserRow {
    pub user_id: Uuid,
    pub nickname: String,
    pub username: String,
    pub bio: String,
    pub post_count: i64,
    pub comment_count: i64,
    pub score: i64,
    pub total_count: i64,
}

impl SearchUserRow {
    fn into_item(self, query: &str) -> SearchResultItem {
        let summary = format!(
            "活跃作者，{} 篇帖子，{} 条评论",
            self.post_count.max(0),
            self.comment_count.max(0)
        );
        SearchResultItem {
            id: self.user_id.to_string(),
            kind: SearchResultKind::User,
            title_highlighted: highlight(&self.nickname, query),
            summary_highlighted: highlight(&summary, query),
            title: self.nickname,
            summary,
            category_name: None,
            tags: Vec::new(),
            author_name: self.username,
            score: self.score,
            url: format!("/users/{}", self.user_id),
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
        let include_source_results = query.category.is_none() && query.tag.is_none();
        let sort = match query.sort {
            SearchSort::Relevance => "relevance",
            SearchSort::Latest => "latest",
            SearchSort::Hot => "hot",
        };
        let limit = if include_source_results {
            (query.page * query.page_size) as i64
        } else {
            query.page_size as i64
        };
        let offset = if include_source_results {
            0
        } else {
            ((query.page - 1) * query.page_size) as i64
        };
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

        let post_total = rows.first().map(|row| row.total_count).unwrap_or(0).max(0) as usize;
        let mut items = rows
            .into_iter()
            .map(|row| row.into_item(&query.q))
            .collect::<Vec<_>>();
        let mut total = post_total;

        if include_source_results {
            let (tag_total, tag_items) = Self::search_tags(pool, &query, limit).await?;
            let (user_total, user_items) = Self::search_users(pool, &query, limit).await?;
            total += tag_total + user_total;
            items.extend(tag_items);
            items.extend(user_items);
            sort_search_items(&mut items, query.sort);
            items = items
                .into_iter()
                .skip((query.page - 1) * query.page_size)
                .take(query.page_size)
                .collect();
        }

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

    async fn search_tags(
        pool: &sqlx::PgPool,
        query: &SearchQuery,
        limit: i64,
    ) -> sqlx::Result<(usize, Vec<SearchResultItem>)> {
        let pattern = format!("%{}%", query.q);
        let rows = sqlx::query_as!(
            SearchTagRow,
            r#"
select
    t.tag_id,
    t.name,
    count(p.post_id)::bigint as "post_count!",
    (
        count(p.post_id) * 10
        + case when t.name ilike $1 then 100 else 0 end
    )::bigint as "score!",
    count(*) over() as "total_count!"
from tags t
left join post_tags pt on pt.tag_id = t.tag_id
left join posts p on p.post_id = pt.post_id and p.status = 'published'
where $2 = '' or t.name ilike $1
group by t.tag_id, t.name
order by 4 desc, t.name asc
limit $3
"#,
            pattern,
            query.q,
            limit
        )
        .fetch_all(pool)
        .await?;

        let total = rows.first().map(|row| row.total_count).unwrap_or(0).max(0) as usize;
        let items = rows
            .into_iter()
            .map(|row| row.into_item(&query.q))
            .collect();
        Ok((total, items))
    }

    async fn search_users(
        pool: &sqlx::PgPool,
        query: &SearchQuery,
        limit: i64,
    ) -> sqlx::Result<(usize, Vec<SearchResultItem>)> {
        let pattern = format!("%{}%", query.q);
        let rows = sqlx::query_as!(
            SearchUserRow,
            r#"
select
    u.user_id,
    u.nickname,
    u.username,
    u.bio,
    (
        select count(*)
        from posts p
        where p.author_id = u.user_id
          and p.status = 'published'
    )::bigint as "post_count!",
    (
        select count(*)
        from comments c
        where c.author_id = u.user_id
          and c.status = 'visible'
    )::bigint as "comment_count!",
    (
        (
            select count(*)
            from posts p
            where p.author_id = u.user_id
              and p.status = 'published'
        ) * 20
        + (
            select count(*)
            from comments c
            where c.author_id = u.user_id
              and c.status = 'visible'
        ) * 5
        + case when u.nickname ilike $1 then 100 else 0 end
        + case when u.username ilike $1 then 40 else 0 end
        + case when u.bio ilike $1 then 20 else 0 end
    )::bigint as "score!",
    count(*) over() as "total_count!"
from users u
where u.status = 'active'
  and ($2 = '' or u.nickname ilike $1 or u.username ilike $1 or u.bio ilike $1)
order by 7 desc, u.created_at desc, u.user_id desc
limit $3
"#,
            pattern,
            query.q,
            limit
        )
        .fetch_all(pool)
        .await?;

        let total = rows.first().map(|row| row.total_count).unwrap_or(0).max(0) as usize;
        let items = rows
            .into_iter()
            .map(|row| row.into_item(&query.q))
            .collect();
        Ok((total, items))
    }
}

pub struct ElasticsearchSearchRepository {
    client: Elasticsearch,
    index: String,
}

impl ElasticsearchSearchRepository {
    pub fn from_url(url: &str, index: impl Into<String>) -> Result<Self, String> {
        let transport = Transport::single_node(url)
            .map_err(|error| format!("Elasticsearch transport 初始化失败: {error}"))?;
        Ok(Self {
            client: Elasticsearch::new(transport),
            index: index.into(),
        })
    }

    pub async fn search_posts(&self, query: SearchQuery) -> Result<SearchResultPage, String> {
        let query = query.normalized();
        let index = self.index.as_str();
        let response = self
            .client
            .search(SearchParts::Index(&[index]))
            .from(((query.page - 1) * query.page_size) as i64)
            .size(query.page_size as i64)
            .body(Self::build_post_search_body(&query))
            .send()
            .await
            .map_err(|error| format!("Elasticsearch 搜索请求失败: {error}"))?;

        let status = response.status_code();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|error| format!("读取错误响应失败: {error}"));
            return Err(format!("Elasticsearch 搜索失败 status={status}: {body}"));
        }

        let body = response
            .json::<Value>()
            .await
            .map_err(|error| format!("Elasticsearch 搜索响应解析失败: {error}"))?;
        Self::parse_post_search_response(query, body)
    }

    pub fn build_post_search_body(query: &SearchQuery) -> Value {
        let mut filters = Vec::new();
        if let Some(category) = &query.category {
            filters.push(json!({ "term": { "category_name": category } }));
        }
        if let Some(tag) = &query.tag {
            filters.push(json!({ "term": { "tags": tag } }));
        }

        let must = if query.q.is_empty() {
            vec![json!({ "match_all": {} })]
        } else {
            vec![json!({
                "multi_match": {
                    "query": query.q,
                    "fields": [
                        "title^4",
                        "summary^2",
                        "name^3",
                        "nickname^3",
                        "username^2",
                        "body",
                        "bio",
                        "tags",
                        "category_name",
                        "author_name"
                    ],
                    "type": "best_fields",
                    "operator": "and"
                }
            })]
        };

        json!({
            "track_total_hits": true,
            "query": {
                "bool": {
                    "must": must,
                    "filter": filters
                }
            },
            "highlight": {
                "pre_tags": ["<mark>"],
                "post_tags": ["</mark>"],
                "fields": {
                    "title": {},
                    "name": {},
                    "nickname": {},
                    "username": {},
                    "summary": {},
                    "bio": {},
                    "body": {}
                }
            },
            "sort": elasticsearch_sort(query.sort)
        })
    }

    pub fn parse_post_search_response(
        query: SearchQuery,
        response: Value,
    ) -> Result<SearchResultPage, String> {
        let hits = response
            .get("hits")
            .ok_or_else(|| "Elasticsearch 响应缺少 hits".to_string())?;
        let total = hits
            .get("total")
            .and_then(|total| {
                total
                    .get("value")
                    .and_then(Value::as_u64)
                    .or_else(|| total.as_u64())
            })
            .unwrap_or(0) as usize;
        let hit_rows = hits
            .get("hits")
            .and_then(Value::as_array)
            .ok_or_else(|| "Elasticsearch 响应缺少 hits.hits 数组".to_string())?;

        let items = hit_rows
            .iter()
            .map(|hit| parse_search_hit(hit, &query))
            .collect::<Result<Vec<_>, _>>()?;

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

fn elasticsearch_sort(sort: SearchSort) -> Value {
    match sort {
        SearchSort::Relevance => json!([{ "_score": { "order": "desc" } }]),
        SearchSort::Latest => {
            json!([{ "published_at": { "order": "desc" } }, { "_score": { "order": "desc" } }])
        }
        SearchSort::Hot => {
            json!([{ "hot_score": { "order": "desc" } }, { "_score": { "order": "desc" } }])
        }
    }
}

fn sort_search_items(items: &mut [SearchResultItem], sort: SearchSort) {
    match sort {
        SearchSort::Relevance | SearchSort::Hot => {
            items.sort_by(|left, right| {
                right
                    .score
                    .cmp(&left.score)
                    .then_with(|| result_kind_rank(left.kind).cmp(&result_kind_rank(right.kind)))
                    .then_with(|| left.title.cmp(&right.title))
            });
        }
        SearchSort::Latest => {
            items.sort_by(|left, right| {
                result_kind_rank(left.kind)
                    .cmp(&result_kind_rank(right.kind))
                    .then_with(|| right.score.cmp(&left.score))
                    .then_with(|| left.title.cmp(&right.title))
            });
        }
    }
}

fn result_kind_rank(kind: SearchResultKind) -> u8 {
    match kind {
        SearchResultKind::Post => 0,
        SearchResultKind::Tag => 1,
        SearchResultKind::User => 2,
    }
}

fn parse_search_hit(hit: &Value, query: &SearchQuery) -> Result<SearchResultItem, String> {
    let source = hit
        .get("_source")
        .ok_or_else(|| "Elasticsearch hit 缺少 _source".to_string())?;
    match source.get("kind").and_then(Value::as_str).unwrap_or("post") {
        "tag" => parse_tag_hit(hit, source, query),
        "user" => parse_user_hit(hit, source, query),
        _ => parse_post_hit(hit, source, query),
    }
}

fn parse_post_hit(
    hit: &Value,
    source: &Value,
    query: &SearchQuery,
) -> Result<SearchResultItem, String> {
    let id = hit
        .get("_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "Elasticsearch hit 缺少 _id".to_string())?
        .to_string();
    let title = string_field(source, "title");
    let summary = string_field(source, "summary");
    let tags = source
        .get("tags")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    Ok(SearchResultItem {
        id: id.clone(),
        kind: SearchResultKind::Post,
        title_highlighted: first_highlight(hit, "title")
            .unwrap_or_else(|| highlight(&title, &query.q)),
        summary_highlighted: first_highlight(hit, "summary")
            .unwrap_or_else(|| highlight(&summary, &query.q)),
        title,
        summary,
        category_name: source
            .get("category_name")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        tags,
        author_name: source
            .get("author_name")
            .and_then(Value::as_str)
            .or_else(|| source.get("author_id").and_then(Value::as_str))
            .unwrap_or("未知用户")
            .to_string(),
        score: source
            .get("hot_score")
            .and_then(Value::as_i64)
            .or_else(|| {
                hit.get("_score")
                    .and_then(Value::as_f64)
                    .map(|score| score as i64)
            })
            .unwrap_or_default(),
        url: format!("/posts/{id}"),
    })
}

fn parse_tag_hit(
    hit: &Value,
    source: &Value,
    query: &SearchQuery,
) -> Result<SearchResultItem, String> {
    let name = string_field(source, "name");
    if name.is_empty() {
        return Err("Elasticsearch tag hit 缺少 name".to_string());
    }
    let post_count = number_field(source, "post_count");
    let summary = format!("{} 个主题使用该标签", post_count.max(0));
    let highlighted_name =
        first_highlight(hit, "name").unwrap_or_else(|| highlight(&name, &query.q));

    Ok(SearchResultItem {
        id: name.clone(),
        kind: SearchResultKind::Tag,
        title_highlighted: format!("#{highlighted_name}"),
        summary_highlighted: highlight(&summary, &query.q),
        title: format!("#{name}"),
        summary,
        category_name: None,
        tags: vec![name.clone()],
        author_name: "标签".to_string(),
        score: hit_score(hit).unwrap_or(post_count),
        url: format!("/search?tag={name}"),
    })
}

fn parse_user_hit(
    hit: &Value,
    source: &Value,
    query: &SearchQuery,
) -> Result<SearchResultItem, String> {
    let id = hit
        .get("_id")
        .and_then(Value::as_str)
        .or_else(|| source.get("user_id").and_then(Value::as_str))
        .ok_or_else(|| "Elasticsearch user hit 缺少 _id".to_string())?
        .to_string();
    let nickname = string_field(source, "nickname");
    if nickname.is_empty() {
        return Err("Elasticsearch user hit 缺少 nickname".to_string());
    }
    let username = string_field(source, "username");
    let post_count = number_field(source, "post_count");
    let comment_count = number_field(source, "comment_count");
    let bio = string_field(source, "bio");
    let summary = if bio.is_empty() {
        format!(
            "活跃作者，{} 篇帖子，{} 条评论",
            post_count.max(0),
            comment_count.max(0)
        )
    } else {
        format!(
            "{}，{} 篇帖子，{} 条评论",
            bio,
            post_count.max(0),
            comment_count.max(0)
        )
    };

    Ok(SearchResultItem {
        id: id.clone(),
        kind: SearchResultKind::User,
        title_highlighted: first_highlight(hit, "nickname")
            .or_else(|| first_highlight(hit, "username"))
            .unwrap_or_else(|| highlight(&nickname, &query.q)),
        summary_highlighted: first_highlight(hit, "bio")
            .unwrap_or_else(|| highlight(&summary, &query.q)),
        title: nickname,
        summary,
        category_name: None,
        tags: Vec::new(),
        author_name: username,
        score: hit_score(hit).unwrap_or(post_count * 20 + comment_count * 5),
        url: format!("/users/{id}"),
    })
}

fn string_field(source: &Value, key: &str) -> String {
    source
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn number_field(source: &Value, key: &str) -> i64 {
    source.get(key).and_then(Value::as_i64).unwrap_or_default()
}

fn hit_score(hit: &Value) -> Option<i64> {
    hit.get("_score")
        .and_then(Value::as_f64)
        .map(|score| score as i64)
}

fn first_highlight(hit: &Value, key: &str) -> Option<String> {
    hit.get("highlight")?
        .get(key)?
        .as_array()?
        .first()?
        .as_str()
        .map(ToString::to_string)
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
