#[test]
fn app_shell_contract_lists_primary_routes() {
    let routes = post::app::primary_routes();

    assert!(routes.contains(&"/"));
    assert!(routes.contains(&"/posts/new"));
    assert!(routes.contains(&"/login"));
    assert!(routes.contains(&"/admin"));
}

#[test]
fn phase1_schema_contains_prd_core_tables() {
    let schema = include_str!("../migrations/202606100001_phase1.sql");

    for table in [
        "users",
        "sessions",
        "roles",
        "permissions",
        "posts",
        "post_contents",
        "comments",
        "post_likes",
        "post_favorites",
        "follows",
        "notifications",
        "announcements",
        "files",
        "reports",
        "audit_logs",
    ] {
        assert!(schema.contains(&format!("create table {table}")), "missing {table}");
    }
}

#[test]
fn post_summary_contract_matches_homepage_requirements() {
    let summary = post::domain::posts::PostSummary::sample();

    assert!(!summary.title.is_empty());
    assert!(!summary.author_name.is_empty());
    assert!(summary.view_count >= 0);
    assert!(summary.comment_count >= 0);
    assert!(summary.like_count >= 0);
    assert!(summary.favorite_count >= 0);
}

#[test]
fn compose_declares_required_prd_services() {
    let compose = include_str!("../docker-compose.yml");

    for service in ["postgres", "redis", "nats", "rustfs", "elasticsearch"] {
        assert!(compose.contains(&format!("{service}:")), "missing {service}");
    }
}

#[test]
fn home_seed_content_exposes_forum_workflow() {
    let text = post::app::home_seed_text();

    for required in ["推荐", "最新", "热门", "发布帖子", "评论", "管理端"] {
        assert!(text.contains(required), "missing {required}");
    }
}

#[test]
fn local_styles_cover_rendered_forum_shell() {
    let css = include_str!("../style/main.css");

    for selector in [".navbar", ".btn", ".grid", ".tabs", ".badge", ".table"] {
        assert!(css.contains(selector), "missing {selector}");
    }
}

#[test]
fn forum_store_supports_core_forum_workflow() {
    let store = post::state::ForumStore::seeded();
    let user = store.demo_user();

    let detail = store
        .create_post(
            user.user_id,
            post::domain::posts::CreatePostRequest {
                title: "从 PRD 到 Leptos 论坛".to_string(),
                markdown: "# 设计\n<script>alert(1)</script>".to_string(),
                summary: "实现一个可发布、可评论、可互动的论坛主链路。".to_string(),
                category_name: Some("Leptos".to_string()),
                tag_names: vec!["rust".to_string(), "forum".to_string()],
                publish: true,
            },
        )
        .expect("create post");

    assert_eq!(detail.summary.author_name, user.nickname);
    assert!(detail.sanitized_html.contains("&lt;script&gt;"));
    assert!(!detail.sanitized_html.contains("<script>"));

    let comment = store
        .add_comment(
            user.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: detail.summary.post_id,
                parent_comment_id: None,
                content: "第一条评论".to_string(),
            },
        )
        .expect("create comment");

    assert_eq!(comment.content, "第一条评论");
    assert_eq!(store.comments_for_post(detail.summary.post_id).unwrap().len(), 1);

    let liked = store.toggle_post_like(user.user_id, detail.summary.post_id).unwrap();
    assert!(liked.active);
    assert_eq!(liked.count, 1);

    let favorited = store.toggle_post_favorite(user.user_id, detail.summary.post_id).unwrap();
    assert!(favorited.active);
    assert_eq!(favorited.count, 1);

    let author = detail.summary.author_id;
    let followed = store.follow_user(user.user_id, author).unwrap_err();
    assert_eq!(followed.to_string(), "请求冲突: 不能关注自己");
}
