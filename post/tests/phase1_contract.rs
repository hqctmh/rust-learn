#[test]
fn app_shell_contract_lists_primary_routes() {
    let routes = post::app::primary_routes();

    assert!(routes.contains(&"/"));
    assert!(routes.contains(&"/posts/new"));
    assert!(routes.contains(&"/login"));
    assert!(routes.contains(&"/register"));
    assert!(routes.contains(&"/admin"));
}

#[test]
fn homepage_component_loads_data_through_server_state() {
    let home_page = std::fs::read_to_string("src/pages/home.rs").expect("read home page source");
    let page_data = std::fs::read_to_string("src/page_data.rs").expect("read page data source");

    assert!(
        home_page.contains("load_home_page"),
        "home page should load data through a server function backed by AppState"
    );
    assert!(
        !home_page.contains("dense_workbench_home("),
        "home page must not render the design mock directly"
    );
    assert!(
        page_data.contains("expect_context::<AppState>()"),
        "page data server functions should consume the AppState provided by the Axum/Leptos integration"
    );
    assert!(
        page_data.contains(".home_page("),
        "home page server function should delegate to AppState::home_page"
    );
}

#[test]
fn primary_ssr_pages_load_operational_data_through_server_state() {
    let pages = [
        (
            "src/pages/search.rs",
            "load_search_page",
            "search_dense_workbench(",
        ),
        (
            "src/pages/admin.rs",
            "load_admin_dashboard",
            "admin_dashboard_demo(",
        ),
        (
            "src/pages/notifications.rs",
            "load_notifications_page",
            "notification_demo_center(",
        ),
    ];
    let page_data = std::fs::read_to_string("src/page_data.rs").expect("read page data source");

    for (path, loader, forbidden_mock) in pages {
        let source = std::fs::read_to_string(path).expect("read page source");
        assert!(
            source.contains(loader),
            "{path} should load data through {loader}"
        );
        assert!(
            !source.contains(forbidden_mock),
            "{path} must not render the design mock directly"
        );
    }

    assert!(page_data.contains(".search("));
    assert!(page_data.contains(".admin_dashboard("));
    assert!(page_data.contains(".notification_center("));
}

#[test]
fn search_page_preserves_url_query_parameters() {
    let source = std::fs::read_to_string("src/pages/search.rs").expect("read search page source");

    assert!(
        source.contains("use_query_map"),
        "search page should read query parameters from the router"
    );
    assert!(
        source.contains("search_query_from_params"),
        "search page should convert URL params into SearchQuery"
    );
    assert!(
        !source.contains("q: \"sqlx\".to_string()"),
        "search page must not hardcode the search keyword"
    );
    assert!(
        !source.contains("href=\"/search?q=sqlx"),
        "search filter links must preserve the current keyword instead of hardcoding sqlx"
    );
}

#[test]
fn home_page_preserves_url_query_parameters_for_filters_and_pagination() {
    let source = std::fs::read_to_string("src/pages/home.rs").expect("read home page source");

    assert!(
        source.contains("use_query_map"),
        "home page should read query parameters from the router"
    );
    assert!(
        source.contains("home_query_from_params"),
        "home page should convert URL params into HomeQuery"
    );
    assert!(
        source.contains("home_href"),
        "home page filter and pager links should preserve query state"
    );
    assert!(
        !source.contains("Resource::new(|| HomeQuery::default()"),
        "home page must not load only the default query"
    );
    assert!(
        !source.contains("href=\"/\""),
        "home page tabs, filters, and pager should not reset all query parameters"
    );
}

#[test]
fn post_detail_page_loads_route_post_and_comments_through_server_state() {
    let source =
        std::fs::read_to_string("src/pages/post_detail.rs").expect("read post detail page source");
    let page_data = std::fs::read_to_string("src/page_data.rs").expect("read page data source");

    assert!(
        source.contains("use_params_map"),
        "post detail page should read the /posts/:id route parameter"
    );
    assert!(
        source.contains("load_post_detail_page"),
        "post detail page should load data through a server function"
    );
    assert!(
        !source.contains("Leptos + Axum 构建全栈论坛的项目结构"),
        "post detail page must not hardcode the article title"
    );
    assert!(
        !source.contains("CommentItem author=\"hello-rust\""),
        "post detail page must not hardcode comments"
    );
    assert!(page_data.contains("PostDetailPageData"));
    assert!(page_data.contains(".post_detail("));
    assert!(page_data.contains(".comments_for_post("));
}

#[test]
fn user_space_pages_load_route_user_through_server_state() {
    let source =
        std::fs::read_to_string("src/pages/user_space.rs").expect("read user space page source");
    let page_data = std::fs::read_to_string("src/page_data.rs").expect("read page data source");

    assert!(
        source.contains("use_params_map"),
        "user profile page should read the /users/:id route parameter"
    );
    assert!(
        source.contains("load_user_space_page"),
        "user space pages should load data through a server function"
    );
    assert!(
        !source.contains("demo_profile("),
        "user space pages must not render demo profile data"
    );
    assert!(
        !source.contains("demo_posts("),
        "user space pages must not render demo post data"
    );
    assert!(page_data.contains("load_user_space_page"));
    assert!(page_data.contains(".user_space("));
}

#[test]
fn auth_contract_supports_register_current_session_and_logout() {
    let store = post::state::ForumStore::seeded();
    let session = store
        .register(post::domain::auth::RegisterRequest {
            username: "new-user".to_string(),
            password: "password".to_string(),
            nickname: "New User".to_string(),
        })
        .expect("register user");

    assert_eq!(session.user.username, "new-user");
    assert_eq!(session.user.nickname, "New User");
    assert!(!session.user.is_admin);

    let duplicate = store.register(post::domain::auth::RegisterRequest {
        username: "new-user".to_string(),
        password: "password".to_string(),
        nickname: "Another".to_string(),
    });
    assert!(duplicate.is_err());

    let current = store
        .current_session(session.session_id)
        .expect("current session");
    assert_eq!(current.user.user_id, session.user.user_id);

    let logged_out = store.logout(session.session_id).expect("logout");
    assert_eq!(logged_out.session_id, session.session_id);
    assert!(store.current_session(session.session_id).is_err());
}

#[test]
fn auth_service_normalizes_credentials_and_builds_sessions() {
    let login = post::services::auth::AuthService::normalize_login("  member  ", " password ")
        .expect("normalize login");
    assert_eq!(login.username, "member");
    assert_eq!(login.password, "password");

    let registration = post::services::auth::AuthService::normalize_registration(
        post::domain::auth::RegisterRequest {
            username: "  new-member  ".to_string(),
            password: " secret ".to_string(),
            nickname: "  New Member  ".to_string(),
        },
    )
    .expect("normalize registration");
    assert_eq!(registration.username, "new-member");
    assert_eq!(registration.password, "secret");
    assert_eq!(registration.nickname, "New Member");

    let user_id = uuid::Uuid::from_u128(901);
    let user = post::services::auth::AuthService::build_registered_user(user_id, registration);
    assert_eq!(user.user_id, user_id);
    assert_eq!(user.username, "new-member");
    assert_eq!(user.nickname, "New Member");
    assert!(!user.is_admin);

    let login_user = post::services::auth::AuthService::build_login_user(
        uuid::Uuid::from_u128(902),
        &login.username,
    );
    assert_eq!(login_user.nickname, "member");
    assert!(!login_user.is_admin);

    post::services::auth::AuthService::validate_password_match("password", &login.password)
        .expect("password matches");
    assert!(
        post::services::auth::AuthService::validate_password_match("other", &login.password)
            .is_err()
    );

    let now = time::OffsetDateTime::now_utc();
    let session = post::services::auth::AuthService::build_session(
        uuid::Uuid::from_u128(903),
        user.clone(),
        now,
    );
    assert_eq!(session.user, user);
    assert_eq!(session.expires_at, now + time::Duration::days(7));
    post::services::auth::AuthService::validate_session_active(session.expires_at, now)
        .expect("session active");
    assert!(
        post::services::auth::AuthService::validate_session_active(
            now - time::Duration::seconds(1),
            now
        )
        .is_err()
    );
}

#[test]
fn auth_routes_are_registered() {
    let primary = post::app::primary_routes();
    assert!(primary.contains(&"/register"));

    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/register"));
    assert!(routes.contains(&"/api/logout"));
    assert!(routes.contains(&"/api/session/{session_id}"));
}

#[test]
fn author_post_contract_supports_draft_autosave_update_and_own_delete() {
    let store = post::state::ForumStore::seeded();
    let author = store.login("author-crud", "password").expect("author").user;
    let other = store.login("other-crud", "password").expect("other").user;

    let draft = store
        .autosave_draft(
            author.user_id,
            post::domain::posts::AutosaveDraftRequest {
                post_id: None,
                title: "草稿自动保存标题".to_string(),
                markdown: "## 草稿\n<script>alert(1)</script>".to_string(),
                summary: "".to_string(),
                category_name: Some("教程".to_string()),
                tag_names: vec!["leptos".to_string(), "draft".to_string()],
            },
        )
        .expect("autosave draft");

    assert_eq!(draft.status, post::domain::posts::PostStatus::Draft);
    assert_eq!(draft.summary.author_id, author.user_id);
    assert!(draft.summary.published_at.is_none());
    assert!(draft.sanitized_html.contains("&lt;script&gt;"));
    assert!(
        store
            .list_posts()
            .iter()
            .all(|post| post.post_id != draft.summary.post_id)
    );

    let updated = store
        .update_post(
            author.user_id,
            draft.summary.post_id,
            post::domain::posts::UpdatePostRequest {
                title: "发布后的 Markdown 帖子".to_string(),
                markdown: "# 发布\n支持表格、引用和代码块。".to_string(),
                summary: "作者可以编辑自己的草稿并发布。".to_string(),
                category_name: Some("经验分享".to_string()),
                tag_names: vec!["markdown".to_string(), "leptos".to_string()],
                publish: true,
            },
        )
        .expect("publish draft");

    assert_eq!(updated.status, post::domain::posts::PostStatus::Published);
    assert_eq!(updated.summary.title, "发布后的 Markdown 帖子");
    assert_eq!(updated.summary.category_name.as_deref(), Some("经验分享"));
    assert!(updated.summary.published_at.is_some());
    assert!(
        store
            .list_posts()
            .iter()
            .any(|post| post.post_id == updated.summary.post_id)
    );

    assert!(
        store
            .update_post(
                other.user_id,
                updated.summary.post_id,
                post::domain::posts::UpdatePostRequest {
                    title: "越权编辑".to_string(),
                    markdown: "不允许".to_string(),
                    summary: "不允许".to_string(),
                    category_name: None,
                    tag_names: vec![],
                    publish: true,
                },
            )
            .is_err()
    );
    assert!(
        store
            .delete_own_post(other.user_id, updated.summary.post_id)
            .is_err()
    );

    let deleted = store
        .delete_own_post(author.user_id, updated.summary.post_id)
        .expect("delete own post");
    assert_eq!(deleted.status, post::domain::posts::PostStatus::Deleted);
    assert!(
        store
            .list_posts()
            .iter()
            .all(|post| post.post_id != updated.summary.post_id)
    );
}

#[test]
fn post_authoring_service_normalizes_and_updates_post_details() {
    let author_id = uuid::Uuid::from_u128(801);
    let post_id = uuid::Uuid::from_u128(802);
    let author = post::domain::auth::SessionUser {
        user_id: author_id,
        username: "authoring-user".to_string(),
        nickname: "AuthoringUser".to_string(),
        avatar_url: Some("/avatar.png".to_string()),
        is_admin: false,
    };
    let now = time::OffsetDateTime::now_utc();

    let invalid = post::services::posts::PostAuthoringService::build_post(
        post_id,
        &author,
        post::domain::posts::CreatePostRequest {
            title: "   ".to_string(),
            markdown: "正文".to_string(),
            summary: "".to_string(),
            category_name: None,
            tag_names: vec![],
            publish: false,
        },
        now,
    );
    assert!(invalid.is_err());

    let mut draft = post::services::posts::PostAuthoringService::build_post(
        post_id,
        &author,
        post::domain::posts::CreatePostRequest {
            title: "  Markdown 草稿  ".to_string(),
            markdown: "## 草稿标题\n<script>alert(1)</script>".to_string(),
            summary: "".to_string(),
            category_name: Some("  教程  ".to_string()),
            tag_names: vec![
                "#Leptos".to_string(),
                "leptos".to_string(),
                " SQLx ".to_string(),
            ],
            publish: false,
        },
        now,
    )
    .expect("build draft");

    assert_eq!(draft.summary.title, "Markdown 草稿");
    assert_eq!(draft.summary.summary, "草稿标题");
    assert_eq!(draft.summary.category_name.as_deref(), Some("教程"));
    assert_eq!(draft.summary.tags, vec!["leptos", "sqlx"]);
    assert_eq!(draft.status, post::domain::posts::PostStatus::Draft);
    assert!(draft.summary.published_at.is_none());
    assert!(draft.sanitized_html.contains("&lt;script&gt;"));

    post::services::posts::PostAuthoringService::apply_update(
        &mut draft,
        post::domain::posts::UpdatePostRequest {
            title: "发布标题".to_string(),
            markdown: "# 发布正文".to_string(),
            summary: "发布摘要".to_string(),
            category_name: Some("经验分享".to_string()),
            tag_names: vec!["Rust".to_string(), "#rust".to_string()],
            publish: true,
        },
        now,
    )
    .expect("publish draft");

    assert_eq!(draft.summary.title, "发布标题");
    assert_eq!(draft.summary.summary, "发布摘要");
    assert_eq!(draft.summary.tags, vec!["rust"]);
    assert_eq!(draft.status, post::domain::posts::PostStatus::Published);
    assert_eq!(draft.summary.published_at, Some(now));
}

#[test]
fn author_post_routes_are_registered() {
    let primary = post::app::primary_routes();
    assert!(primary.contains(&"/posts/sample/edit"));

    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/posts/drafts/autosave"));
    assert!(routes.contains(&"/api/posts/{post_id}/update"));
    assert!(routes.contains(&"/api/posts/{post_id}/delete"));
}

#[test]
fn protected_api_handlers_require_explicit_actor_identity() {
    let api = include_str!("../src/api.rs");

    assert!(
        api.contains("fn require_user_id"),
        "protected API handlers must share an explicit actor identity guard"
    );
    assert!(
        api.contains("ForumError::Unauthorized"),
        "missing Unauthorized mapping for protected API identity failures"
    );
    assert!(
        !api.contains("demo_user()"),
        "API handlers must not silently impersonate the demo user for protected actions"
    );
}

#[test]
fn protected_api_handlers_resolve_actor_from_session_id() {
    let api = include_str!("../src/api.rs");

    assert!(
        api.contains("session_id: Option<Uuid>"),
        "protected API identity params should accept session_id"
    );
    assert!(
        api.contains("async fn require_actor_id"),
        "protected API handlers should share async session-aware actor resolution"
    );
    assert!(
        api.contains(".current_session(session_id).await"),
        "session_id must be verified through AppState::current_session before resolving actor"
    );
    assert!(
        api.contains("require_actor_id(&state"),
        "protected API handlers should call the session-aware actor guard"
    );
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
        assert!(
            schema.contains(&format!("create table {table}")),
            "missing {table}"
        );
    }
}

#[test]
fn sqlx_post_repository_contract_maps_homepage_post_rows() {
    let sql = post::repositories::posts::PostgresPostRepository::published_summaries_sql();
    assert!(!sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "from posts p",
        "join users u on u.user_id = p.author_id",
        "left join categories c on c.category_id = p.category_id",
        "left join post_tags pt on pt.post_id = p.post_id",
        "left join tags t on t.tag_id = pt.tag_id",
        "where p.status = 'published'",
        "order by p.is_pinned desc",
        "limit $1 offset $2",
    ] {
        assert!(
            sql.to_ascii_lowercase().contains(required),
            "missing SQL fragment: {required}"
        );
    }

    let row = post::repositories::posts::PostSummaryRow {
        post_id: uuid::Uuid::from_u128(1401),
        title: "PostgreSQL 持久化边界".to_string(),
        summary: "首页帖子列表来自 SQLx repository。".to_string(),
        author_id: uuid::Uuid::from_u128(1402),
        author_name: "mah".to_string(),
        author_avatar_url: Some("/avatar.png".to_string()),
        category_name: Some("教程".to_string()),
        tags: vec!["leptos".to_string(), "sqlx".to_string()],
        view_count: 128,
        comment_count: 6,
        like_count: 19,
        favorite_count: 8,
        published_at: Some(time::OffsetDateTime::now_utc()),
    };
    let summary: post::domain::posts::PostSummary = row.into();
    assert_eq!(summary.title, "PostgreSQL 持久化边界");
    assert_eq!(summary.author_name, "mah");
    assert_eq!(summary.tags, vec!["leptos", "sqlx"]);
    assert_eq!(summary.view_count, 128);
}

#[test]
fn sqlx_post_detail_repository_contract_maps_post_detail_row() {
    let sql = post::repositories::posts::PostgresPostRepository::post_detail_sql();
    assert!(!sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "from posts p",
        "join users u on u.user_id = p.author_id",
        "join post_contents pc on pc.post_id = p.post_id",
        "left join categories c on c.category_id = p.category_id",
        "left join post_tags pt on pt.post_id = p.post_id",
        "left join tags t on t.tag_id = pt.tag_id",
        "where p.post_id = $1",
        "group by",
        "limit 1",
    ] {
        assert!(
            sql.to_ascii_lowercase().contains(required),
            "missing post detail SQL fragment: {required}"
        );
    }

    let row = post::repositories::posts::PostDetailRow {
        post_id: uuid::Uuid::from_u128(1601),
        title: "详情持久化".to_string(),
        summary: "帖子详情来自 PostgreSQL。".to_string(),
        author_id: uuid::Uuid::from_u128(1602),
        author_name: "mah".to_string(),
        author_avatar_url: None,
        category_name: Some("教程".to_string()),
        tags: vec!["sqlx".to_string(), "leptos".to_string()],
        view_count: 99,
        comment_count: 4,
        like_count: 8,
        favorite_count: 3,
        published_at: Some(time::OffsetDateTime::now_utc()),
        markdown: "# 详情".to_string(),
        sanitized_html: "<h1>详情</h1>".to_string(),
        status: "published".to_string(),
    };
    let detail: post::domain::posts::PostDetail = row.into();
    assert_eq!(detail.summary.title, "详情持久化");
    assert_eq!(detail.summary.tags, vec!["sqlx", "leptos"]);
    assert_eq!(detail.markdown, "# 详情");
    assert_eq!(detail.sanitized_html, "<h1>详情</h1>");
    assert_eq!(detail.status, post::domain::posts::PostStatus::Published);
    assert!(!detail.liked_by_me);
    assert!(!detail.favorited_by_me);
    assert!(!detail.following_author);
}

#[test]
fn sqlx_comment_repository_contract_maps_comment_tree_rows() {
    let sql = post::repositories::comments::PostgresCommentRepository::comments_for_post_sql();
    assert!(!sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "from comments c",
        "join users u on u.user_id = c.author_id",
        "join posts p on p.post_id = c.post_id",
        "where c.post_id = $1",
        "order by c.created_at asc",
    ] {
        assert!(
            sql.to_ascii_lowercase().contains(required),
            "missing comment SQL fragment: {required}"
        );
    }

    let post_id = uuid::Uuid::from_u128(1701);
    let author_id = uuid::Uuid::from_u128(1702);
    let viewer_id = uuid::Uuid::from_u128(1703);
    let parent_id = uuid::Uuid::from_u128(1704);
    let reply_id = uuid::Uuid::from_u128(1705);
    let deleted_id = uuid::Uuid::from_u128(1706);
    let now = time::OffsetDateTime::now_utc();
    let rows = vec![
        post::repositories::comments::CommentRow {
            comment_id: parent_id,
            post_id,
            parent_comment_id: None,
            author_id,
            author_name: "作者".to_string(),
            content: "主评论".to_string(),
            status: "visible".to_string(),
            like_count: 3,
            created_at: now,
            post_author_id: author_id,
        },
        post::repositories::comments::CommentRow {
            comment_id: reply_id,
            post_id,
            parent_comment_id: Some(parent_id),
            author_id: viewer_id,
            author_name: "读者".to_string(),
            content: "回复".to_string(),
            status: "visible".to_string(),
            like_count: 1,
            created_at: now,
            post_author_id: author_id,
        },
        post::repositories::comments::CommentRow {
            comment_id: deleted_id,
            post_id,
            parent_comment_id: None,
            author_id: viewer_id,
            author_name: "读者".to_string(),
            content: "需要隐藏".to_string(),
            status: "deleted".to_string(),
            like_count: 0,
            created_at: now,
            post_author_id: author_id,
        },
    ];

    let comments =
        post::repositories::comments::PostgresCommentRepository::build_comment_tree(rows);
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].comment_id, parent_id);
    assert_eq!(comments[0].content, "主评论");
    assert!(comments[0].author_reply);
    assert_eq!(comments[0].replies.len(), 1);
    assert_eq!(comments[0].replies[0].comment_id, reply_id);
    assert!(!comments[0].replies[0].author_reply);
    assert_eq!(comments[1].comment_id, deleted_id);
    assert!(comments[1].deleted);
    assert_eq!(comments[1].content, "该评论已被删除");
}

#[test]
fn sqlx_auth_repository_contract_maps_users_and_sessions() {
    let find_user_sql =
        post::repositories::auth::PostgresAuthRepository::find_user_by_username_sql();
    assert!(!find_user_sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "select",
        "user_id",
        "username",
        "password_hash",
        "nickname",
        "avatar_url",
        "status",
        "is_admin",
        "from users",
        "where username = $1",
        "limit 1",
    ] {
        assert!(
            find_user_sql.to_ascii_lowercase().contains(required),
            "missing find user SQL fragment: {required}"
        );
    }

    let insert_user_sql = post::repositories::auth::PostgresAuthRepository::insert_user_sql();
    assert!(!insert_user_sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "insert into users",
        "user_id",
        "username",
        "password_hash",
        "nickname",
        "avatar_url",
        "is_admin",
        "returning",
        "status",
    ] {
        assert!(
            insert_user_sql.to_ascii_lowercase().contains(required),
            "missing insert user SQL fragment: {required}"
        );
    }

    let insert_session_sql = post::repositories::auth::PostgresAuthRepository::insert_session_sql();
    assert!(!insert_session_sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "insert into sessions",
        "session_id",
        "user_id",
        "token_hash",
        "expires_at",
    ] {
        assert!(
            insert_session_sql.to_ascii_lowercase().contains(required),
            "missing insert session SQL fragment: {required}"
        );
    }

    let find_session_sql = post::repositories::auth::PostgresAuthRepository::find_session_sql();
    assert!(!find_session_sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "from sessions s",
        "join users u on u.user_id = s.user_id",
        "where s.session_id = $1",
        "limit 1",
    ] {
        assert!(
            find_session_sql.to_ascii_lowercase().contains(required),
            "missing find session SQL fragment: {required}"
        );
    }

    let delete_session_sql = post::repositories::auth::PostgresAuthRepository::delete_session_sql();
    assert!(
        delete_session_sql
            .to_ascii_lowercase()
            .contains("delete from sessions")
    );
    assert!(
        delete_session_sql
            .to_ascii_lowercase()
            .contains("where session_id = $1")
    );

    let user_row = post::repositories::auth::UserAuthRow {
        user_id: uuid::Uuid::from_u128(1501),
        username: "auth-user".to_string(),
        password_hash: "hash".to_string(),
        nickname: "Auth User".to_string(),
        avatar_url: Some("/auth.png".to_string()),
        status: "active".to_string(),
        is_admin: false,
    };
    assert!(!user_row.is_disabled());
    assert_eq!(user_row.password_hash, "hash");
    let session_user = user_row.session_user();
    assert_eq!(session_user.username, "auth-user");
    assert_eq!(session_user.nickname, "Auth User");
    assert_eq!(session_user.avatar_url, Some("/auth.png".to_string()));
    assert!(!session_user.is_admin);

    let disabled_user_row = post::repositories::auth::UserAuthRow {
        status: "disabled".to_string(),
        ..user_row
    };
    assert!(disabled_user_row.is_disabled());

    let expires_at = time::OffsetDateTime::now_utc();
    let session_row = post::repositories::auth::SessionAuthRow {
        session_id: uuid::Uuid::from_u128(1502),
        user_id: uuid::Uuid::from_u128(1503),
        username: "session-user".to_string(),
        nickname: "Session User".to_string(),
        avatar_url: None,
        status: "active".to_string(),
        is_admin: true,
        expires_at,
    };
    let session = session_row.session();
    assert_eq!(session.session_id, uuid::Uuid::from_u128(1502));
    assert_eq!(session.user.username, "session-user");
    assert!(session.user.is_admin);
    assert_eq!(session.expires_at, expires_at);
    assert!(!session_row.is_disabled());
}

#[test]
fn sqlx_home_sidebar_repository_contract_maps_homepage_sidebar_rows() {
    let categories_sql =
        post::repositories::home::PostgresHomeRepository::homepage_categories_sql();
    assert!(!categories_sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "from categories c",
        "left join posts p on p.category_id = c.category_id",
        "p.status = 'published'",
        "group by c.category_id",
        "order by c.sort_order asc",
    ] {
        assert!(
            categories_sql.to_ascii_lowercase().contains(required),
            "missing categories SQL fragment: {required}"
        );
    }

    let hot_tags_sql = post::repositories::home::PostgresHomeRepository::hot_tags_sql();
    assert!(!hot_tags_sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "from tags t",
        "join post_tags pt on pt.tag_id = t.tag_id",
        "join posts p on p.post_id = pt.post_id",
        "where p.status = 'published'",
        "limit $1",
    ] {
        assert!(
            hot_tags_sql.to_ascii_lowercase().contains(required),
            "missing hot tags SQL fragment: {required}"
        );
    }

    let announcements_sql = post::repositories::home::PostgresHomeRepository::announcements_sql();
    assert!(!announcements_sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "from announcements",
        "where status = 'published'",
        "starts_at is null",
        "ends_at is null",
        "order by is_pinned desc",
        "limit $1",
    ] {
        assert!(
            announcements_sql.to_ascii_lowercase().contains(required),
            "missing announcements SQL fragment: {required}"
        );
    }

    let active_authors_sql = post::repositories::home::PostgresHomeRepository::active_authors_sql();
    assert!(!active_authors_sql.to_ascii_lowercase().contains("select *"));
    for required in [
        "from users u",
        "join comments c on c.author_id = u.user_id",
        "c.status = 'visible'",
        "group by u.user_id",
        "limit $1",
    ] {
        assert!(
            active_authors_sql.to_ascii_lowercase().contains(required),
            "missing active authors SQL fragment: {required}"
        );
    }

    let category: post::domain::home::HomeCategory = post::repositories::home::HomeCategoryRow {
        name: "教程".to_string(),
        count: 34,
        color: "green".to_string(),
    }
    .into();
    assert_eq!(category.name, "教程");
    assert_eq!(category.count, 34);
    assert_eq!(category.color, "green");

    let tag: post::domain::home::HomeTag = post::repositories::home::HomeTagRow {
        name: "leptos".to_string(),
        count: 132,
    }
    .into();
    assert_eq!(tag.name, "leptos");
    assert_eq!(tag.count, 132);

    let announcement: post::domain::home::HomeAnnouncement =
        post::repositories::home::HomeAnnouncementRow {
            title: "Leptos 0.6 正式发布".to_string(),
            date_label: "5 月 20 日".to_string(),
        }
        .into();
    assert_eq!(announcement.title, "Leptos 0.6 正式发布");
    assert_eq!(announcement.date_label, "5 月 20 日");

    let author: post::domain::home::HomeActiveAuthor =
        post::repositories::home::HomeActiveAuthorRow {
            name: "张晨".to_string(),
            avatar_label: "张".to_string(),
            reply_count_label: "1.2k 条回复".to_string(),
        }
        .into();
    assert_eq!(author.name, "张晨");
    assert_eq!(author.avatar_label, "张");
    assert_eq!(author.reply_count_label, "1.2k 条回复");
}

#[test]
fn sqlx_repository_execution_uses_checked_macros() {
    let sources = [
        ("auth.rs", include_str!("../src/repositories/auth.rs")),
        ("posts.rs", include_str!("../src/repositories/posts.rs")),
        (
            "comments.rs",
            include_str!("../src/repositories/comments.rs"),
        ),
        ("files.rs", include_str!("../src/repositories/files.rs")),
        ("home.rs", include_str!("../src/repositories/home.rs")),
        ("reports.rs", include_str!("../src/repositories/reports.rs")),
        (
            "announcements.rs",
            include_str!("../src/repositories/announcements.rs"),
        ),
        (
            "taxonomy.rs",
            include_str!("../src/repositories/taxonomy.rs"),
        ),
        (
            "moderation.rs",
            include_str!("../src/repositories/moderation.rs"),
        ),
        ("users.rs", include_str!("../src/repositories/users.rs")),
        (
            "notifications.rs",
            include_str!("../src/repositories/notifications.rs"),
        ),
        ("rbac.rs", include_str!("../src/repositories/rbac.rs")),
        ("search.rs", include_str!("../src/repositories/search.rs")),
        (
            "user_admin.rs",
            include_str!("../src/repositories/user_admin.rs"),
        ),
    ];

    for (file, source) in sources {
        assert!(
            !source.contains("query_as::<"),
            "{file} must use sqlx::query_as! macro instead of runtime query_as"
        );
        assert!(
            !source.contains("sqlx::query("),
            "{file} must use sqlx::query! macro instead of runtime query"
        );
    }
}

#[tokio::test]
async fn app_state_auth_runtime_supports_postgres_mode_and_demo_fallback() {
    let demo_state = post::state::AppState {
        db: None,
        forum: post::state::ForumStore::seeded(),
    };
    assert!(!demo_state.uses_postgres_auth());

    let session = demo_state
        .register(post::domain::auth::RegisterRequest {
            username: "runtime-user".to_string(),
            password: "password".to_string(),
            nickname: "Runtime User".to_string(),
        })
        .await
        .expect("register through app state");
    assert_eq!(session.user.username, "runtime-user");
    assert_eq!(session.user.nickname, "Runtime User");

    let current = demo_state
        .current_session(session.session_id)
        .await
        .expect("current session through app state");
    assert_eq!(current.user.user_id, session.user.user_id);

    let login = demo_state
        .login("runtime-user", "password")
        .await
        .expect("login through app state");
    assert_eq!(login.user.username, "runtime-user");

    let logout = demo_state
        .logout(session.session_id)
        .await
        .expect("logout through app state");
    assert_eq!(logout.session_id, session.session_id);

    let lazy_pool =
        sqlx::PgPool::connect_lazy("postgres://post:post@localhost:5433/post").expect("lazy pool");
    let postgres_state = post::state::AppState {
        db: Some(lazy_pool),
        forum: post::state::ForumStore::seeded(),
    };
    assert!(postgres_state.uses_postgres_auth());
}

#[tokio::test]
async fn app_state_post_list_runtime_supports_demo_fallback() {
    let state = post::state::AppState {
        db: None,
        forum: post::state::ForumStore::seeded(),
    };

    let posts = state
        .list_posts()
        .await
        .expect("list posts through app state");
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].title, "Rust 异步任务的边界设计");
    assert_eq!(posts[0].author_name, "mah");
}

#[tokio::test]
async fn app_state_create_post_persists_to_postgres_and_reads_back() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let session = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("author-{suffix}"),
            password: "password".to_string(),
            nickname: format!("作者{suffix}"),
        })
        .await
        .expect("register postgres author");

    let title = format!("SQLx 宏持久化发帖 {suffix}");
    let detail = state
        .create_post(
            session.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("  {title}  "),
                markdown: "# 发布正文\n<script>alert(1)</script>".to_string(),
                summary: "".to_string(),
                category_name: Some("教程".to_string()),
                tag_names: vec![
                    "SQLx".to_string(),
                    "#leptos".to_string(),
                    "sqlx".to_string(),
                ],
                publish: true,
            },
        )
        .await
        .expect("create post through app state");

    assert_eq!(detail.summary.title, title);
    assert_eq!(detail.summary.author_id, session.user.user_id);
    assert_eq!(detail.summary.category_name.as_deref(), Some("教程"));
    assert_eq!(detail.summary.tags, vec!["sqlx", "leptos"]);
    assert!(detail.summary.published_at.is_some());
    assert!(detail.sanitized_html.contains("&lt;script&gt;"));

    let listed = state.list_posts().await.expect("list posts");
    assert!(
        listed
            .iter()
            .any(|post| post.post_id == detail.summary.post_id)
    );

    let loaded = state
        .post_detail(detail.summary.post_id)
        .await
        .expect("load persisted post detail");
    assert_eq!(loaded.summary.title, title);
    assert_eq!(loaded.markdown, "# 发布正文\n<script>alert(1)</script>");
    assert_eq!(loaded.status, post::domain::posts::PostStatus::Published);
}

#[tokio::test]
async fn app_state_add_comment_persists_to_postgres_and_updates_post_count() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("post-{suffix}"),
            password: "password".to_string(),
            nickname: format!("楼主{suffix}"),
        })
        .await
        .expect("register post author");
    let commenter = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("comment-{suffix}"),
            password: "password".to_string(),
            nickname: format!("评论{suffix}"),
        })
        .await
        .expect("register commenter");

    let detail = state
        .create_post(
            author.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("评论持久化帖子 {suffix}"),
                markdown: "正文".to_string(),
                summary: "评论计数需要持久化".to_string(),
                category_name: Some("问题".to_string()),
                tag_names: vec!["comments".to_string()],
                publish: true,
            },
        )
        .await
        .expect("create post");

    let comment = state
        .add_comment(
            commenter.user.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: detail.summary.post_id,
                parent_comment_id: None,
                content: "  这条评论需要写入 PostgreSQL  ".to_string(),
            },
        )
        .await
        .expect("add postgres comment");

    assert_eq!(comment.post_id, detail.summary.post_id);
    assert_eq!(comment.author_id, commenter.user.user_id);
    assert_eq!(comment.author_name, commenter.user.nickname);
    assert_eq!(comment.content, "这条评论需要写入 PostgreSQL");
    assert!(!comment.author_reply);

    let comments = state
        .comments_for_post(detail.summary.post_id)
        .await
        .expect("list comments");
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].comment_id, comment.comment_id);

    let loaded = state
        .post_detail(detail.summary.post_id)
        .await
        .expect("load post detail");
    assert_eq!(loaded.summary.comment_count, 1);
}

#[tokio::test]
async fn app_state_comment_reactions_and_delete_persist_to_postgres() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("cpost-{suffix}"),
            password: "password".to_string(),
            nickname: format!("楼主{suffix}"),
        })
        .await
        .expect("register post author");
    let commenter = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("cauthor-{suffix}"),
            password: "password".to_string(),
            nickname: format!("评论者{suffix}"),
        })
        .await
        .expect("register commenter");
    let viewer = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("cviewer-{suffix}"),
            password: "password".to_string(),
            nickname: format!("读者{suffix}"),
        })
        .await
        .expect("register viewer");
    let post = state
        .create_post(
            author.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("评论互动帖子 {suffix}"),
                markdown: "正文".to_string(),
                summary: "评论互动需要落库".to_string(),
                category_name: Some("问题".to_string()),
                tag_names: vec!["comment".to_string()],
                publish: true,
            },
        )
        .await
        .expect("create post");
    let comment = state
        .add_comment(
            commenter.user.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: post.summary.post_id,
                parent_comment_id: None,
                content: "可以点赞和删除的评论".to_string(),
            },
        )
        .await
        .expect("create comment");

    let liked = state
        .toggle_comment_like(viewer.user.user_id, comment.comment_id)
        .await
        .expect("like comment");
    assert!(liked.active);
    assert_eq!(liked.count, 1);

    let comments = state
        .comments_for_post(post.summary.post_id)
        .await
        .expect("list liked comment");
    assert_eq!(comments[0].like_count, 1);

    let unliked = state
        .toggle_comment_like(viewer.user.user_id, comment.comment_id)
        .await
        .expect("unlike comment");
    assert!(!unliked.active);
    assert_eq!(unliked.count, 0);

    assert!(
        state
            .delete_own_comment(viewer.user.user_id, comment.comment_id)
            .await
            .is_err()
    );
    let deleted = state
        .delete_own_comment(commenter.user.user_id, comment.comment_id)
        .await
        .expect("delete own comment");
    assert!(deleted.deleted);
    assert_eq!(deleted.content, "该评论已被删除");

    let loaded = state
        .post_detail(post.summary.post_id)
        .await
        .expect("load post after comment delete");
    assert_eq!(loaded.summary.comment_count, 0);

    let public_comments = state
        .comments_for_post(post.summary.post_id)
        .await
        .expect("public comments");
    assert_eq!(public_comments[0].content, "该评论已被删除");
}

#[tokio::test]
async fn app_state_create_report_persists_to_postgres() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool.clone()),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("rpost-{suffix}"),
            password: "password".to_string(),
            nickname: format!("楼主{suffix}"),
        })
        .await
        .expect("register post author");
    let reporter = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("reporter-{suffix}"),
            password: "password".to_string(),
            nickname: format!("举报{suffix}"),
        })
        .await
        .expect("register reporter");
    let post = state
        .create_post(
            author.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("待举报帖子 {suffix}"),
                markdown: "正文".to_string(),
                summary: "需要被举报的帖子".to_string(),
                category_name: Some("问题".to_string()),
                tag_names: vec!["report".to_string()],
                publish: true,
            },
        )
        .await
        .expect("create post");

    let report = state
        .create_report(
            reporter.user.user_id,
            post::domain::reports::CreateReportRequest {
                target_type: post::domain::reports::ReportTargetType::Post,
                target_id: post.summary.post_id,
                reason: "内容违规".to_string(),
                description: Some("包含不合适内容".to_string()),
            },
        )
        .await
        .expect("create postgres report");

    assert_eq!(report.reporter_id, reporter.user.user_id);
    assert_eq!(report.target_id, post.summary.post_id);
    assert_eq!(report.target_title, Some(post.summary.title.clone()));
    assert_eq!(report.status, post::domain::reports::ReportStatus::Pending);

    let row = sqlx::query!(
        r#"
select
    target_type,
    target_id,
    reporter_id,
    reason,
    description,
    status
from reports
where report_id = $1
"#,
        report.report_id
    )
    .fetch_one(&pool)
    .await
    .expect("load report row");

    assert_eq!(row.target_type, "post");
    assert_eq!(row.target_id, post.summary.post_id);
    assert_eq!(row.reporter_id, reporter.user.user_id);
    assert_eq!(row.reason, "内容违规");
    assert_eq!(row.description, "包含不合适内容");
    assert_eq!(row.status, "pending");
}

#[tokio::test]
async fn app_state_admin_report_list_and_handle_persist_to_postgres() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool.clone()),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let admin = post::repositories::auth::PostgresAuthRepository::insert_user(
        &pool,
        uuid::Uuid::new_v4(),
        &format!("admin-report-{suffix}"),
        "password",
        &format!("管理员{suffix}"),
        None,
        true,
    )
    .await
    .expect("insert admin");
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("report-author-{suffix}"),
            password: "password".to_string(),
            nickname: format!("作者{suffix}"),
        })
        .await
        .expect("register author");
    let reporter = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("report-reader-{suffix}"),
            password: "password".to_string(),
            nickname: format!("读者{suffix}"),
        })
        .await
        .expect("register reporter");
    let post = state
        .create_post(
            author.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("后台举报帖子 {suffix}"),
                markdown: "正文".to_string(),
                summary: "后台需要处理的举报".to_string(),
                category_name: Some("问题".to_string()),
                tag_names: vec!["report-admin".to_string()],
                publish: true,
            },
        )
        .await
        .expect("create post");
    let report = state
        .create_report(
            reporter.user.user_id,
            post::domain::reports::CreateReportRequest {
                target_type: post::domain::reports::ReportTargetType::Post,
                target_id: post.summary.post_id,
                reason: "垃圾内容".to_string(),
                description: None,
            },
        )
        .await
        .expect("create report");

    let reports = state
        .list_reports(admin.user_id)
        .await
        .expect("list reports from postgres");
    let listed = reports
        .iter()
        .find(|item| item.report_id == report.report_id)
        .expect("new report listed");
    assert_eq!(listed.status, post::domain::reports::ReportStatus::Pending);
    assert_eq!(listed.reporter_name, reporter.user.nickname);
    assert_eq!(listed.target_title, Some(post.summary.title.clone()));

    let handled = state
        .handle_report(
            admin.user_id,
            report.report_id,
            post::domain::reports::HandleReportRequest {
                status: post::domain::reports::ReportStatus::Handled,
                note: "已下线违规内容".to_string(),
            },
        )
        .await
        .expect("handle report");
    assert_eq!(handled.status, post::domain::reports::ReportStatus::Handled);
    assert_eq!(handled.handler_id, Some(admin.user_id));
    assert_eq!(handled.handler_name, Some(admin.nickname.clone()));
    assert_eq!(handled.handle_note.as_deref(), Some("已下线违规内容"));
    assert!(handled.handled_at.is_some());

    let listed_after_handle = state
        .list_reports(admin.user_id)
        .await
        .expect("list handled reports");
    let persisted = listed_after_handle
        .iter()
        .find(|item| item.report_id == report.report_id)
        .expect("handled report listed");
    assert_eq!(
        persisted.status,
        post::domain::reports::ReportStatus::Handled
    );
    assert_eq!(persisted.handle_note.as_deref(), Some("已下线违规内容"));
}

#[tokio::test]
async fn app_state_announcement_admin_and_public_flows_persist_to_postgres() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool.clone()),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let admin = post::repositories::auth::PostgresAuthRepository::insert_user(
        &pool,
        uuid::Uuid::new_v4(),
        &format!("admin-ann-{suffix}"),
        "password",
        &format!("公告管理员{suffix}"),
        None,
        true,
    )
    .await
    .expect("insert admin");
    let reader = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("ann-reader-{suffix}"),
            password: "password".to_string(),
            nickname: format!("公告读者{suffix}"),
        })
        .await
        .expect("register reader");

    let draft = state
        .create_announcement(
            admin.user_id,
            post::domain::announcements::CreateAnnouncementRequest {
                title: format!("数据库公告 {suffix}"),
                content: "公告内容需要进入 PostgreSQL。".to_string(),
                announcement_type: "system".to_string(),
                pinned: true,
                effective_at: None,
                expires_at: None,
                audience: post::domain::announcements::AnnouncementAudience::AllUsers,
            },
        )
        .await
        .expect("create announcement");
    assert_eq!(
        draft.status,
        post::domain::announcements::AnnouncementStatus::Draft
    );

    let admin_list = state
        .list_admin_announcements(admin.user_id)
        .await
        .expect("list admin announcements");
    assert!(
        admin_list
            .iter()
            .any(|item| item.announcement_id == draft.announcement_id)
    );

    let published = state
        .publish_announcement(admin.user_id, draft.announcement_id)
        .await
        .expect("publish announcement");
    assert_eq!(
        published.status,
        post::domain::announcements::AnnouncementStatus::Published
    );
    assert!(published.published_at.is_some());

    let public_items = state.public_announcements().await;
    assert!(
        public_items
            .iter()
            .any(|item| item.announcement_id == draft.announcement_id)
    );

    let read_state = state
        .mark_announcement_read(reader.user.user_id, draft.announcement_id)
        .await
        .expect("mark announcement read");
    assert_eq!(read_state.announcement_id, draft.announcement_id);
    assert_eq!(read_state.user_id, reader.user.user_id);
    assert!(read_state.read);

    let withdrawn = state
        .withdraw_announcement(admin.user_id, draft.announcement_id)
        .await
        .expect("withdraw announcement");
    assert_eq!(
        withdrawn.status,
        post::domain::announcements::AnnouncementStatus::Withdrawn
    );
    assert!(withdrawn.withdrawn_at.is_some());

    let public_after_withdraw = state.public_announcements().await;
    assert!(
        public_after_withdraw
            .iter()
            .all(|item| item.announcement_id != draft.announcement_id)
    );
}

#[tokio::test]
async fn app_state_taxonomy_admin_and_public_flows_persist_to_postgres() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool.clone()),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let admin = post::repositories::auth::PostgresAuthRepository::insert_user(
        &pool,
        uuid::Uuid::new_v4(),
        &format!("admin-tax-{suffix}"),
        "password",
        &format!("分类管理员{suffix}"),
        None,
        true,
    )
    .await
    .expect("insert admin");
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("tax-author-{suffix}"),
            password: "password".to_string(),
            nickname: format!("分类作者{suffix}"),
        })
        .await
        .expect("register author");

    let category_name = format!("源码解析-{suffix}");
    let category = state
        .create_category(
            admin.user_id,
            post::domain::taxonomy::CreateCategoryRequest {
                name: category_name.clone(),
                color: "#0064E0".to_string(),
                sort_order: 7,
            },
        )
        .await
        .expect("create category");
    assert_eq!(category.name, category_name);
    assert_eq!(category.color, "#0064E0");

    let source_tag = state
        .create_tag(
            admin.user_id,
            post::domain::taxonomy::CreateTagRequest {
                name: format!("merge-source-{suffix}"),
                sort_order: 3,
            },
        )
        .await
        .expect("create source tag");
    let source_tag = state
        .update_tag(
            admin.user_id,
            source_tag.tag_id,
            post::domain::taxonomy::UpdateTagRequest {
                name: None,
                sort_order: Some(3),
                enabled: Some(true),
                use_count: Some(7),
            },
        )
        .await
        .expect("seed source count");
    let target_tag = state
        .create_tag(
            admin.user_id,
            post::domain::taxonomy::CreateTagRequest {
                name: format!("merge-target-{suffix}"),
                sort_order: 2,
            },
        )
        .await
        .expect("create target tag");
    let target_tag = state
        .update_tag(
            admin.user_id,
            target_tag.tag_id,
            post::domain::taxonomy::UpdateTagRequest {
                name: None,
                sort_order: Some(2),
                enabled: Some(true),
                use_count: Some(5),
            },
        )
        .await
        .expect("seed target count");

    let post = state
        .create_post(
            author.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("分类标签持久化 {suffix}"),
                markdown: "正文".to_string(),
                summary: "分类和标签需要进入 PostgreSQL。".to_string(),
                category_name: Some(category.name.clone()),
                tag_names: vec![source_tag.name.clone()],
                publish: true,
            },
        )
        .await
        .expect("create taxonomy post");
    assert_eq!(
        post.summary.category_name.as_deref(),
        Some(category.name.as_str())
    );

    let public_categories = state.public_categories().await;
    let public_category = public_categories
        .iter()
        .find(|item| item.category_id == category.category_id)
        .expect("public category");
    assert_eq!(public_category.post_count, 1);

    let updated_category = state
        .update_category(
            admin.user_id,
            category.category_id,
            post::domain::taxonomy::UpdateCategoryRequest {
                name: Some(format!("源码阅读-{suffix}")),
                color: Some("#0A7CFF".to_string()),
                sort_order: Some(1),
                enabled: Some(true),
            },
        )
        .await
        .expect("update category");
    assert_eq!(updated_category.color, "#0A7CFF");
    assert_eq!(updated_category.sort_order, 1);

    let merged = state
        .merge_tag(
            admin.user_id,
            source_tag.tag_id,
            post::domain::taxonomy::MergeTagRequest {
                target_tag_id: target_tag.tag_id,
            },
        )
        .await
        .expect("merge tags");
    assert_eq!(
        merged.use_count,
        source_tag.use_count + target_tag.use_count
    );

    let public_tags = state.public_tags().await;
    assert!(
        public_tags
            .iter()
            .any(|item| item.tag_id == target_tag.tag_id && item.use_count == merged.use_count)
    );
    assert!(
        public_tags
            .iter()
            .all(|item| item.tag_id != source_tag.tag_id)
    );

    let disabled_category = state
        .disable_category(admin.user_id, category.category_id)
        .await
        .expect("disable category");
    assert!(!disabled_category.enabled);
    assert!(
        state
            .public_categories()
            .await
            .iter()
            .all(|item| item.category_id != category.category_id)
    );

    let deleted_tag = state
        .delete_tag(admin.user_id, target_tag.tag_id)
        .await
        .expect("delete tag");
    assert!(!deleted_tag.enabled);
    assert_eq!(deleted_tag.use_count, 0);
    assert!(
        state
            .public_tags()
            .await
            .iter()
            .all(|item| item.tag_id != target_tag.tag_id)
    );
}

#[tokio::test]
async fn app_state_content_moderation_persists_to_postgres() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool.clone()),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let admin = post::repositories::auth::PostgresAuthRepository::insert_user(
        &pool,
        uuid::Uuid::new_v4(),
        &format!("admin-mod-{suffix}"),
        "password",
        &format!("内容管理员{suffix}"),
        None,
        true,
    )
    .await
    .expect("insert admin");
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("mod-author-{suffix}"),
            password: "password".to_string(),
            nickname: format!("审核作者{suffix}"),
        })
        .await
        .expect("register author");
    let commenter = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("mod-commenter-{suffix}"),
            password: "password".to_string(),
            nickname: format!("审核评论者{suffix}"),
        })
        .await
        .expect("register commenter");
    let post = state
        .create_post(
            author.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("后台内容审核 {suffix}"),
                markdown: "正文".to_string(),
                summary: "后台需要能审核帖子和评论。".to_string(),
                category_name: Some("问题".to_string()),
                tag_names: vec!["moderation".to_string()],
                publish: true,
            },
        )
        .await
        .expect("create post");
    let comment = state
        .add_comment(
            commenter.user.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: post.summary.post_id,
                parent_comment_id: None,
                content: "这条评论需要后台审核".to_string(),
            },
        )
        .await
        .expect("create comment");

    assert!(state.admin_posts(commenter.user.user_id).await.is_err());
    let admin_posts = state.admin_posts(admin.user_id).await.expect("admin posts");
    assert!(
        admin_posts
            .iter()
            .any(|item| item.post_id == post.summary.post_id)
    );

    let offline = state
        .take_down_post(admin.user_id, post.summary.post_id)
        .await
        .expect("take down post");
    assert_eq!(offline.status, post::domain::posts::PostStatus::Offline);
    assert!(
        state
            .list_posts()
            .await
            .expect("list posts after offline")
            .iter()
            .all(|item| item.post_id != post.summary.post_id)
    );

    let restored = state
        .restore_post(admin.user_id, post.summary.post_id)
        .await
        .expect("restore post");
    assert_eq!(restored.status, post::domain::posts::PostStatus::Published);
    assert!(
        state
            .list_posts()
            .await
            .expect("list posts after restore")
            .iter()
            .any(|item| item.post_id == post.summary.post_id)
    );

    let pinned = state
        .pin_post(admin.user_id, post.summary.post_id)
        .await
        .expect("pin post");
    assert!(pinned.pinned);
    let unpinned = state
        .unpin_post(admin.user_id, post.summary.post_id)
        .await
        .expect("unpin post");
    assert!(!unpinned.pinned);

    let admin_comments = state
        .admin_comments(admin.user_id)
        .await
        .expect("admin comments");
    assert!(
        admin_comments
            .iter()
            .any(|item| item.comment_id == comment.comment_id)
    );

    let deleted_comment = state
        .delete_comment(admin.user_id, comment.comment_id)
        .await
        .expect("delete comment");
    assert!(deleted_comment.deleted);
    assert_eq!(
        state
            .post_detail(post.summary.post_id)
            .await
            .expect("post after comment delete")
            .summary
            .comment_count,
        0
    );
    let public_comments = state
        .comments_for_post(post.summary.post_id)
        .await
        .expect("public comments after delete");
    assert_eq!(public_comments[0].content, "该评论已被删除");

    let recovered_comment = state
        .recover_comment(admin.user_id, comment.comment_id)
        .await
        .expect("recover comment");
    assert!(!recovered_comment.deleted);
    assert_eq!(
        state
            .post_detail(post.summary.post_id)
            .await
            .expect("post after comment recover")
            .summary
            .comment_count,
        1
    );

    let deleted_post = state
        .delete_post(admin.user_id, post.summary.post_id)
        .await
        .expect("delete post");
    assert_eq!(
        deleted_post.status,
        post::domain::posts::PostStatus::Deleted
    );
    assert!(
        state
            .list_posts()
            .await
            .expect("list posts after delete")
            .iter()
            .all(|item| item.post_id != post.summary.post_id)
    );
}

#[tokio::test]
async fn app_state_user_settings_persist_to_postgres() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let session = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("settings-{suffix}"),
            password: "password".to_string(),
            nickname: format!("设置用户{suffix}"),
        })
        .await
        .expect("register settings user");

    let profile = state
        .update_profile(
            session.user.user_id,
            post::domain::users::UpdateProfileRequest {
                nickname: format!("Profile {suffix}"),
                bio: "专注 Leptos、Axum 和 SQLx 的全栈开发者。".to_string(),
            },
        )
        .await
        .expect("update profile");
    assert_eq!(profile.nickname, format!("Profile {suffix}"));
    assert_eq!(profile.bio, "专注 Leptos、Axum 和 SQLx 的全栈开发者。");

    let avatar = state
        .update_avatar(
            session.user.user_id,
            post::domain::users::UpdateAvatarRequest {
                avatar_url: format!("/uploads/avatars/settings-{suffix}.png"),
            },
        )
        .await
        .expect("update avatar");
    assert_eq!(
        avatar.avatar_url.as_deref(),
        Some(format!("/uploads/avatars/settings-{suffix}.png").as_str())
    );

    assert!(
        state
            .change_password(
                session.user.user_id,
                post::domain::users::ChangePasswordRequest {
                    old_password: "wrong-password".to_string(),
                    new_password: "new-password".to_string(),
                },
            )
            .await
            .is_err()
    );
    state
        .change_password(
            session.user.user_id,
            post::domain::users::ChangePasswordRequest {
                old_password: "password".to_string(),
                new_password: "new-password".to_string(),
            },
        )
        .await
        .expect("change password");
    assert!(
        state
            .login(&format!("settings-{suffix}"), "password")
            .await
            .is_err()
    );
    let logged_in = state
        .login(&format!("settings-{suffix}"), "new-password")
        .await
        .expect("login with new password");
    assert_eq!(logged_in.user.user_id, session.user.user_id);
    assert_eq!(logged_in.user.nickname, format!("Profile {suffix}"));
}

#[tokio::test]
async fn app_state_user_space_aggregates_postgres_activity() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("space-author-{suffix}"),
            password: "password".to_string(),
            nickname: format!("空间作者{suffix}"),
        })
        .await
        .expect("register author");
    let viewer = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("space-viewer-{suffix}"),
            password: "password".to_string(),
            nickname: format!("空间读者{suffix}"),
        })
        .await
        .expect("register viewer");

    let published = state
        .create_post(
            author.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("个人空间公开帖 {suffix}"),
                markdown: "正文".to_string(),
                summary: "公开帖应出现在个人空间。".to_string(),
                category_name: Some("经验分享".to_string()),
                tag_names: vec!["space".to_string()],
                publish: true,
            },
        )
        .await
        .expect("create published post");
    let draft = state
        .autosave_draft(
            author.user.user_id,
            post::domain::posts::AutosaveDraftRequest {
                post_id: None,
                title: format!("个人空间草稿 {suffix}"),
                markdown: "草稿正文".to_string(),
                summary: "草稿应出现在作者空间。".to_string(),
                category_name: Some("教程".to_string()),
                tag_names: vec!["draft-space".to_string()],
            },
        )
        .await
        .expect("create draft");
    state
        .follow_user(viewer.user.user_id, author.user.user_id)
        .await
        .expect("follow author");
    state
        .toggle_post_favorite(viewer.user.user_id, published.summary.post_id)
        .await
        .expect("favorite post");
    state
        .add_comment(
            viewer.user.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: published.summary.post_id,
                parent_comment_id: None,
                content: "收藏备用".to_string(),
            },
        )
        .await
        .expect("add comment");

    let author_space = state
        .user_space(author.user.user_id, Some(viewer.user.user_id))
        .await
        .expect("author space");
    assert_eq!(author_space.profile.user_id, author.user.user_id);
    assert!(author_space.followed_by_viewer);
    assert_eq!(author_space.stats.followers, 1);
    assert!(
        author_space
            .published_posts
            .iter()
            .any(|item| item.post_id == published.summary.post_id)
    );
    assert!(
        author_space
            .draft_posts
            .iter()
            .any(|item| item.post_id == draft.summary.post_id)
    );

    let viewer_space = state
        .user_space(viewer.user.user_id, Some(viewer.user.user_id))
        .await
        .expect("viewer space");
    assert!(viewer_space.is_me);
    assert_eq!(viewer_space.stats.following, 1);
    assert_eq!(viewer_space.following.len(), 1);
    assert!(
        viewer_space
            .favorite_posts
            .iter()
            .any(|item| item.post_id == published.summary.post_id)
    );
    assert!(
        viewer_space
            .comments
            .iter()
            .any(|item| item.post_id == published.summary.post_id && item.content == "收藏备用")
    );
}

#[tokio::test]
async fn app_state_notifications_persist_to_postgres_and_mark_read() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("notify-author-{suffix}"),
            password: "password".to_string(),
            nickname: format!("通知作者{suffix}"),
        })
        .await
        .expect("register author");
    let follower = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("notify-follower-{suffix}"),
            password: "password".to_string(),
            nickname: format!("通知粉丝{suffix}"),
        })
        .await
        .expect("register follower");

    state
        .follow_user(follower.user.user_id, author.user.user_id)
        .await
        .expect("follow author");
    let post = state
        .create_post(
            author.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("通知中心帖子 {suffix}"),
                markdown: "正文".to_string(),
                summary: "发布后通知关注者。".to_string(),
                category_name: Some("公告".to_string()),
                tag_names: vec!["notification".to_string()],
                publish: true,
            },
        )
        .await
        .expect("publish post");

    let follower_center = state
        .notification_center(follower.user.user_id)
        .await
        .expect("follower notifications");
    assert_eq!(follower_center.unread_count, 1);
    assert_eq!(
        follower_center.items[0].notification_type,
        post::domain::notifications::NotificationType::FollowedUserPosted
    );

    state
        .add_comment(
            follower.user.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: post.summary.post_id,
                parent_comment_id: None,
                content: "这篇帖子很有帮助".to_string(),
            },
        )
        .await
        .expect("comment post");
    let author_center = state
        .notification_center(author.user.user_id)
        .await
        .expect("author notifications");
    assert_eq!(author_center.unread_count, 1);
    assert_eq!(
        author_center.items[0].notification_type,
        post::domain::notifications::NotificationType::PostCommented
    );

    let notification_id = author_center.items[0].notification_id;
    let after_read = state
        .mark_notification_read(author.user.user_id, notification_id)
        .await
        .expect("mark read");
    assert_eq!(after_read.unread_count, 0);

    let all_read = state
        .mark_all_notifications_read(follower.user.user_id)
        .await
        .expect("mark all read");
    assert_eq!(all_read.unread_count, 0);
}

#[tokio::test]
async fn app_state_user_admin_persists_to_postgres_and_writes_audit_logs() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool.clone()),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let admin = post::repositories::auth::PostgresAuthRepository::insert_user(
        &pool,
        uuid::Uuid::new_v4(),
        &format!("admin-{suffix}"),
        "password",
        &format!("管理员{suffix}"),
        None,
        true,
    )
    .await
    .expect("insert postgres admin")
    .session_user();
    let member = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("managed-{suffix}"),
            password: "password".to_string(),
            nickname: format!("被管理用户{suffix}"),
        })
        .await
        .expect("register postgres member")
        .user;

    assert!(state.admin_users(member.user_id).await.is_err());
    assert!(
        state
            .admin_users(admin.user_id)
            .await
            .expect("postgres admin users")
            .iter()
            .any(|user| user.user_id == member.user_id && !user.disabled)
    );

    let context = post::domain::user_admin::AuditContext {
        ip: Some("127.0.0.1".to_string()),
        user_agent: Some("postgres-user-admin-test".to_string()),
    };
    let disabled = state
        .disable_user(admin.user_id, member.user_id, context.clone())
        .await
        .expect("disable postgres user");
    assert!(disabled.disabled);
    assert!(state.login(&member.username, "password").await.is_err());

    let enabled = state
        .enable_user(admin.user_id, member.user_id, context.clone())
        .await
        .expect("enable postgres user");
    assert!(!enabled.disabled);
    assert!(state.login(&member.username, "password").await.is_ok());

    let roles = state
        .update_user_roles(
            admin.user_id,
            member.user_id,
            post::domain::user_admin::UpdateUserRolesRequest {
                roles: vec![
                    " Moderator ".to_string(),
                    "operator".to_string(),
                    "moderator".to_string(),
                ],
                context: context.clone(),
            },
        )
        .await
        .expect("update postgres user roles");
    assert_eq!(roles.roles, vec!["moderator", "operator"]);

    let logs = state
        .audit_logs(admin.user_id)
        .await
        .expect("postgres audit logs");
    assert!(logs.iter().any(|entry| {
        entry.actor_id == admin.user_id
            && entry.action == "user.disable"
            && entry.target_id == member.user_id
            && entry.ip.as_deref() == Some("127.0.0.1")
            && entry.user_agent.as_deref() == Some("postgres-user-admin-test")
    }));
    assert!(logs.iter().any(|entry| {
        entry.actor_id == admin.user_id
            && entry.action == "user.roles.update"
            && entry.target_id == member.user_id
            && entry
                .after
                .as_deref()
                .is_some_and(|value| value.contains("moderator"))
    }));
}

#[tokio::test]
async fn app_state_rbac_roles_persist_to_postgres_and_write_audit_logs() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool.clone()),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let admin = post::repositories::auth::PostgresAuthRepository::insert_user(
        &pool,
        uuid::Uuid::new_v4(),
        &format!("rbac-admin-{suffix}"),
        "password",
        &format!("RBAC管理员{suffix}"),
        None,
        true,
    )
    .await
    .expect("insert postgres rbac admin")
    .session_user();
    let member = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("rbac-member-{suffix}"),
            password: "password".to_string(),
            nickname: format!("RBAC成员{suffix}"),
        })
        .await
        .expect("register postgres rbac member")
        .user;

    assert!(state.list_roles(member.user_id).await.is_err());
    assert!(state.list_permissions(member.user_id).await.is_err());

    let permissions = state
        .list_permissions(admin.user_id)
        .await
        .expect("postgres admin permissions");
    for required in ["role:view", "role:create", "role:update", "role:delete"] {
        assert!(
            permissions
                .iter()
                .any(|permission| permission.code == required),
            "missing permission {required}"
        );
    }

    let context = post::domain::user_admin::AuditContext {
        ip: Some("127.0.0.1".to_string()),
        user_agent: Some("postgres-rbac-test".to_string()),
    };
    let reviewer_code = format!("reviewer-{suffix}");
    let reviewer = state
        .create_role(
            admin.user_id,
            post::domain::rbac::CreateRoleRequest {
                code: reviewer_code.clone(),
                name: "内容审核员".to_string(),
                permission_codes: vec![
                    "post:view".to_string(),
                    "comment:view".to_string(),
                    "comment:delete".to_string(),
                ],
                context: context.clone(),
            },
        )
        .await
        .expect("create postgres role");
    assert_eq!(reviewer.code, reviewer_code);
    assert_eq!(reviewer.permissions.len(), 3);

    assert!(
        state
            .create_role(
                admin.user_id,
                post::domain::rbac::CreateRoleRequest {
                    code: reviewer.code.clone(),
                    name: "重复角色".to_string(),
                    permission_codes: vec!["post:view".to_string()],
                    context: context.clone(),
                },
            )
            .await
            .is_err()
    );

    let updated = state
        .update_role(
            admin.user_id,
            &reviewer.code,
            post::domain::rbac::UpdateRoleRequest {
                name: Some("高级审核员".to_string()),
                permission_codes: Some(vec![
                    "post:view".to_string(),
                    "post:update".to_string(),
                    "report:view".to_string(),
                ]),
                context: context.clone(),
            },
        )
        .await
        .expect("update postgres role");
    assert_eq!(updated.name, "高级审核员");
    assert!(
        updated
            .permissions
            .iter()
            .any(|permission| permission.code == "report:view")
    );

    state
        .update_user_roles(
            admin.user_id,
            member.user_id,
            post::domain::user_admin::UpdateUserRolesRequest {
                roles: vec![reviewer.code.clone()],
                context: context.clone(),
            },
        )
        .await
        .expect("assign postgres role");
    assert!(
        state
            .delete_role(admin.user_id, &reviewer.code, context.clone())
            .await
            .is_err()
    );

    let temporary_code = format!("temporary-{suffix}");
    let temporary = state
        .create_role(
            admin.user_id,
            post::domain::rbac::CreateRoleRequest {
                code: temporary_code.clone(),
                name: "临时角色".to_string(),
                permission_codes: vec!["user:view".to_string()],
                context: context.clone(),
            },
        )
        .await
        .expect("create temporary postgres role");
    assert_eq!(temporary.code, temporary_code);
    let deleted = state
        .delete_role(admin.user_id, &temporary.code, context.clone())
        .await
        .expect("delete temporary postgres role");
    assert_eq!(deleted.code, temporary_code);
    assert!(
        state
            .list_roles(admin.user_id)
            .await
            .expect("postgres roles after delete")
            .iter()
            .all(|role| role.code != temporary_code)
    );

    let logs = state
        .audit_logs(admin.user_id)
        .await
        .expect("postgres rbac audit logs");
    assert!(logs.iter().any(|entry| {
        entry.actor_id == admin.user_id
            && entry.action == "role.create"
            && entry.target_type == "role"
            && entry
                .after
                .as_deref()
                .is_some_and(|value| value.contains(&reviewer_code))
    }));
    assert!(logs.iter().any(|entry| {
        entry.actor_id == admin.user_id
            && entry.action == "role.update"
            && entry
                .after
                .as_deref()
                .is_some_and(|value| value.contains("report:view"))
    }));
    assert!(logs.iter().any(|entry| {
        entry.actor_id == admin.user_id
            && entry.action == "role.delete"
            && entry
                .before
                .as_deref()
                .is_some_and(|value| value.contains(&temporary_code))
    }));
}

#[tokio::test]
async fn app_state_search_reads_postgres_posts_with_filters_and_highlights() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let session = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("search-author-{suffix}"),
            password: "password".to_string(),
            nickname: format!("搜索作者{suffix}"),
        })
        .await
        .expect("register postgres search author");
    let marker = format!("search-{suffix}");
    let post = state
        .create_post(
            session.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("Postgres {marker} runtime"),
                markdown: format!("正文里也包含 {marker}，用于验证 PostgreSQL 搜索。"),
                summary: format!("摘要包含 {marker}"),
                category_name: Some("教程".to_string()),
                tag_names: vec![marker.clone()],
                publish: true,
            },
        )
        .await
        .expect("create searchable postgres post");

    let results = state
        .search(post::domain::search::SearchQuery {
            q: marker.clone(),
            category: Some("教程".to_string()),
            tag: Some(marker.clone()),
            sort: post::domain::search::SearchSort::Latest,
            page: 1,
            page_size: 5,
        })
        .await
        .expect("search postgres posts");

    assert_eq!(results.query.q, marker);
    assert!(results.total >= 1);
    let item = results
        .items
        .iter()
        .find(|item| item.id == post.summary.post_id.to_string())
        .expect("search result item");
    assert_eq!(item.kind, post::domain::search::SearchResultKind::Post);
    assert_eq!(item.category_name.as_deref(), Some("教程"));
    assert!(item.tags.contains(&results.query.q));
    assert_eq!(item.author_name, session.user.nickname);
    assert_eq!(item.url, format!("/posts/{}", post.summary.post_id));
    assert!(item.title_highlighted.contains("<mark>"));
    assert!(item.summary_highlighted.contains("<mark>"));
}

#[tokio::test]
async fn app_state_file_upload_persists_to_postgres_and_deduplicates_hash() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let session = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("file-uploader-{suffix}"),
            password: "password".to_string(),
            nickname: format!("上传用户{suffix}"),
        })
        .await
        .expect("register postgres uploader");
    let request = post::domain::files::FileUploadRequest {
        original_filename: format!("cover {suffix}.png"),
        file_size: 128_000,
        mime_type: "image/png".to_string(),
        content_hash: format!("sha256-file-{suffix}"),
        usage: post::domain::files::FileUsage::MarkdownImage,
    };

    let asset = state
        .upload_file(session.user.user_id, request.clone())
        .await
        .expect("upload postgres file");
    assert_eq!(asset.bucket, "post-assets");
    assert!(asset.storage_key.ends_with(&format!("/cover-{suffix}.png")));
    assert_eq!(asset.public_url, format!("/uploads/{}", asset.storage_key));
    assert_eq!(
        asset.markdown_image,
        format!("![cover-{suffix}.png]({})", asset.public_url)
    );

    let duplicate = state
        .upload_file(session.user.user_id, request)
        .await
        .expect("deduplicate postgres file");
    assert_eq!(duplicate.file_id, asset.file_id);
    assert_eq!(duplicate.file_hash, asset.file_hash);

    let invalid = state
        .upload_file(
            session.user.user_id,
            post::domain::files::FileUploadRequest {
                original_filename: "shell.svg".to_string(),
                file_size: 100,
                mime_type: "image/svg+xml".to_string(),
                content_hash: format!("sha256-svg-{suffix}"),
                usage: post::domain::files::FileUsage::MarkdownImage,
            },
        )
        .await;
    assert!(invalid.is_err());
}

#[tokio::test]
async fn app_state_admin_dashboard_aggregates_postgres_runtime_data() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool.clone()),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let admin = post::repositories::auth::PostgresAuthRepository::insert_user(
        &pool,
        uuid::Uuid::new_v4(),
        &format!("dashboard-admin-{suffix}"),
        "password",
        &format!("仪表盘管理员{suffix}"),
        None,
        true,
    )
    .await
    .expect("insert postgres dashboard admin")
    .session_user();
    let member = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("dashboard-member-{suffix}"),
            password: "password".to_string(),
            nickname: format!("仪表盘成员{suffix}"),
        })
        .await
        .expect("register dashboard member")
        .user;
    let post = state
        .create_post(
            member.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("仪表盘统计帖子 {suffix}"),
                markdown: "用于验证后台 dashboard 从 PostgreSQL 聚合。".to_string(),
                summary: "后台 dashboard 统计".to_string(),
                category_name: Some("教程".to_string()),
                tag_names: vec![format!("dashboard-{suffix}")],
                publish: true,
            },
        )
        .await
        .expect("create dashboard post");
    state
        .add_comment(
            admin.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: post.summary.post_id,
                parent_comment_id: None,
                content: format!("dashboard-comment-{suffix}"),
            },
        )
        .await
        .expect("create dashboard comment");

    assert!(state.admin_dashboard(member.user_id).await.is_err());
    let dashboard = state
        .admin_dashboard(admin.user_id)
        .await
        .expect("postgres admin dashboard");

    assert!(dashboard.stats.iter().any(|stat| stat.label == "用户总数"));
    assert!(dashboard.stats.iter().any(|stat| stat.label == "帖子总数"));
    assert!(
        dashboard
            .users
            .iter()
            .any(|user| user.username == member.username && user.status == "正常")
    );
    assert!(
        dashboard
            .moderation_posts
            .iter()
            .any(|item| item.title == post.summary.title && item.author == member.nickname)
    );
    assert!(dashboard.moderation_comments.iter().any(|comment| {
        comment.post_title == post.summary.title
            && comment.content == format!("dashboard-comment-{suffix}")
    }));
    assert!(dashboard.menu.iter().any(|item| item.label == "系统统计"));
    assert!(
        dashboard
            .permissions
            .iter()
            .any(|permission| permission.code == "audit:view")
    );
}

#[tokio::test]
async fn app_state_notification_socket_state_accepts_postgres_users() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let session = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("socket-user-{suffix}"),
            password: "password".to_string(),
            nickname: format!("Socket用户{suffix}"),
        })
        .await
        .expect("register postgres socket user");

    let connected = state
        .connect_notification_socket(session.user.user_id)
        .await
        .expect("connect postgres socket user");
    assert_eq!(connected.user_id, session.user.user_id);
    assert_eq!(connected.online_connections, 1);

    let stats = state
        .notification_connection_stats(session.user.user_id)
        .await
        .expect("postgres socket stats");
    assert_eq!(stats.online_connections, 1);
    assert_eq!(stats.pending_push_count, 0);
    assert!(
        state
            .pending_notification_pushes(session.user.user_id)
            .await
            .expect("pending postgres socket pushes")
            .is_empty()
    );

    let disconnected = state
        .disconnect_notification_socket(session.user.user_id)
        .await
        .expect("disconnect postgres socket user");
    assert_eq!(disconnected.online_connections, 0);
}

#[tokio::test]
async fn app_state_post_reactions_persist_to_postgres_and_update_counts() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("react-post-{suffix}"),
            password: "password".to_string(),
            nickname: format!("作者{suffix}"),
        })
        .await
        .expect("register author");
    let viewer = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("react-user-{suffix}"),
            password: "password".to_string(),
            nickname: format!("读者{suffix}"),
        })
        .await
        .expect("register viewer");
    let post = state
        .create_post(
            author.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("互动持久化帖子 {suffix}"),
                markdown: "正文".to_string(),
                summary: "点赞收藏计数要落库".to_string(),
                category_name: Some("经验分享".to_string()),
                tag_names: vec!["reaction".to_string()],
                publish: true,
            },
        )
        .await
        .expect("create post");

    let liked = state
        .toggle_post_like(viewer.user.user_id, post.summary.post_id)
        .await
        .expect("like post");
    assert!(liked.active);
    assert_eq!(liked.count, 1);

    let favorited = state
        .toggle_post_favorite(viewer.user.user_id, post.summary.post_id)
        .await
        .expect("favorite post");
    assert!(favorited.active);
    assert_eq!(favorited.count, 1);

    let loaded = state
        .post_detail(post.summary.post_id)
        .await
        .expect("load reacted post");
    assert_eq!(loaded.summary.like_count, 1);
    assert_eq!(loaded.summary.favorite_count, 1);

    let unliked = state
        .toggle_post_like(viewer.user.user_id, post.summary.post_id)
        .await
        .expect("unlike post");
    assert!(!unliked.active);
    assert_eq!(unliked.count, 0);
}

#[tokio::test]
async fn app_state_follow_user_persists_to_postgres_and_toggles() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let follower = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("follower-{suffix}"),
            password: "password".to_string(),
            nickname: format!("粉丝{suffix}"),
        })
        .await
        .expect("register follower");
    let followee = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("followee-{suffix}"),
            password: "password".to_string(),
            nickname: format!("作者{suffix}"),
        })
        .await
        .expect("register followee");

    let followed = state
        .follow_user(follower.user.user_id, followee.user.user_id)
        .await
        .expect("follow user");
    assert!(followed.following);
    assert_eq!(followed.follower_id, follower.user.user_id);
    assert_eq!(followed.followee_id, followee.user.user_id);

    let unfollowed = state
        .follow_user(follower.user.user_id, followee.user.user_id)
        .await
        .expect("unfollow user");
    assert!(!unfollowed.following);

    let self_follow = state
        .follow_user(follower.user.user_id, follower.user.user_id)
        .await;
    assert!(self_follow.is_err());
}

#[tokio::test]
async fn app_state_update_and_delete_post_persist_to_postgres() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("edit-{suffix}"),
            password: "password".to_string(),
            nickname: format!("编辑{suffix}"),
        })
        .await
        .expect("register author");
    let created = state
        .create_post(
            author.user.user_id,
            post::domain::posts::CreatePostRequest {
                title: format!("待编辑帖子 {suffix}"),
                markdown: "初始正文".to_string(),
                summary: "初始摘要".to_string(),
                category_name: Some("教程".to_string()),
                tag_names: vec!["before".to_string()],
                publish: true,
            },
        )
        .await
        .expect("create post");

    let updated = state
        .update_post(
            author.user.user_id,
            created.summary.post_id,
            post::domain::posts::UpdatePostRequest {
                title: format!("已编辑帖子 {suffix}"),
                markdown: "# 新正文\n<script>alert(1)</script>".to_string(),
                summary: "更新后的摘要".to_string(),
                category_name: Some("经验分享".to_string()),
                tag_names: vec!["after".to_string(), "SQLx".to_string()],
                publish: true,
            },
        )
        .await
        .expect("update post");

    assert_eq!(updated.summary.title, format!("已编辑帖子 {suffix}"));
    assert_eq!(updated.summary.category_name.as_deref(), Some("经验分享"));
    assert_eq!(updated.summary.tags, vec!["after", "sqlx"]);
    assert!(updated.sanitized_html.contains("&lt;script&gt;"));

    let loaded = state
        .post_detail(created.summary.post_id)
        .await
        .expect("load updated post");
    assert_eq!(loaded.summary.title, updated.summary.title);
    assert_eq!(loaded.markdown, "# 新正文\n<script>alert(1)</script>");

    let deleted = state
        .delete_own_post(author.user.user_id, created.summary.post_id)
        .await
        .expect("delete own post");
    assert_eq!(deleted.status, post::domain::posts::PostStatus::Deleted);

    assert!(state.post_detail(created.summary.post_id).await.is_err());
    let posts = state.list_posts().await.expect("list posts");
    assert!(
        posts
            .iter()
            .all(|post| post.post_id != created.summary.post_id)
    );
}

#[tokio::test]
async fn app_state_autosave_draft_persists_to_postgres_and_updates_existing_draft() {
    let pool = sqlx::PgPool::connect("postgres://post:post@localhost:5433/post")
        .await
        .expect("connect postgres");
    let state = post::state::AppState {
        db: Some(pool),
        forum: post::state::ForumStore::seeded(),
    };

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let author = state
        .register(post::domain::auth::RegisterRequest {
            username: format!("draft-{suffix}"),
            password: "password".to_string(),
            nickname: format!("草稿{suffix}"),
        })
        .await
        .expect("register author");

    let draft = state
        .autosave_draft(
            author.user.user_id,
            post::domain::posts::AutosaveDraftRequest {
                post_id: None,
                title: format!("草稿标题 {suffix}"),
                markdown: "## 草稿正文\n<script>alert(1)</script>".to_string(),
                summary: "".to_string(),
                category_name: Some("教程".to_string()),
                tag_names: vec!["Draft".to_string(), "#SQLx".to_string()],
            },
        )
        .await
        .expect("create postgres draft");

    assert_eq!(draft.status, post::domain::posts::PostStatus::Draft);
    assert!(draft.summary.published_at.is_none());
    assert_eq!(draft.summary.tags, vec!["draft", "sqlx"]);
    assert!(draft.sanitized_html.contains("&lt;script&gt;"));

    let listed = state.list_posts().await.expect("list posts");
    assert!(
        listed
            .iter()
            .all(|post| post.post_id != draft.summary.post_id)
    );

    let updated = state
        .autosave_draft(
            author.user.user_id,
            post::domain::posts::AutosaveDraftRequest {
                post_id: Some(draft.summary.post_id),
                title: format!("更新草稿 {suffix}"),
                markdown: "更新后的正文".to_string(),
                summary: "更新摘要".to_string(),
                category_name: Some("经验分享".to_string()),
                tag_names: vec!["updated".to_string()],
            },
        )
        .await
        .expect("update postgres draft");

    assert_eq!(updated.summary.post_id, draft.summary.post_id);
    assert_eq!(updated.summary.title, format!("更新草稿 {suffix}"));
    assert_eq!(updated.summary.summary, "更新摘要");
    assert_eq!(updated.summary.category_name.as_deref(), Some("经验分享"));
    assert_eq!(updated.summary.tags, vec!["updated"]);

    let loaded = state
        .post_detail(draft.summary.post_id)
        .await
        .expect("load updated draft");
    assert_eq!(loaded.status, post::domain::posts::PostStatus::Draft);
    assert_eq!(loaded.markdown, "更新后的正文");
}

#[tokio::test]
async fn app_state_home_runtime_supports_demo_fallback() {
    let state = post::state::AppState {
        db: None,
        forum: post::state::ForumStore::seeded(),
    };

    let home = state
        .home_page(post::domain::home::HomeQuery::default(), None)
        .await
        .expect("home page through app state");
    assert_eq!(home.topics.len(), 12);
    assert_eq!(home.categories.len(), 6);
    assert_eq!(home.hot_tags.len(), 8);
    assert_eq!(home.pagination.label, "显示 1-12 / 342 个主题");
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
        assert!(
            compose.contains(&format!("{service}:")),
            "missing {service}"
        );
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
fn home_page_data_matches_dense_workbench_design_contract() {
    let store = post::state::ForumStore::seeded();
    let home = store
        .home_page(post::domain::home::HomeQuery::default(), None)
        .expect("home page data");

    assert_eq!(home.topics.len(), 12);
    assert_eq!(home.pagination.page, 1);
    assert_eq!(home.pagination.page_size, 12);
    assert_eq!(home.pagination.total, 342);
    assert_eq!(home.pagination.total_pages, 29);
    assert_eq!(home.pagination.label, "显示 1-12 / 342 个主题");

    let titles: Vec<_> = home
        .topics
        .iter()
        .map(|topic| topic.title.as_str())
        .collect();
    for required in [
        "Leptos 0.6 发布：更快的编译、更小的体积、Signal 优化",
        "新手指南：从 Axum + Leptos + SQLx 搭建全栈应用",
        "站点规则与发帖规范（必读）",
        "从零实现一个简单的 Leptos 组件库",
    ] {
        assert!(titles.contains(&required), "missing topic {required}");
    }

    assert_eq!(home.categories.len(), 6);
    assert_eq!(home.hot_tags.len(), 8);
    assert_eq!(home.announcements.len(), 3);
    assert_eq!(home.active_authors.len(), 5);
}

#[test]
fn home_page_query_supports_tabs_filters_and_pagination_defaults() {
    let store = post::state::ForumStore::seeded();

    let unanswered = store
        .home_page(
            post::domain::home::HomeQuery {
                tab: post::domain::home::HomeTab::Unanswered,
                ..Default::default()
            },
            None,
        )
        .expect("unanswered home page");
    assert!(unanswered.topics.iter().all(|topic| topic.reply_count == 0));

    let leptos = store
        .home_page(
            post::domain::home::HomeQuery {
                tag: Some("leptos".to_string()),
                ..Default::default()
            },
            None,
        )
        .expect("tag filtered home page");
    assert!(
        leptos
            .topics
            .iter()
            .all(|topic| topic.tags.iter().any(|tag| tag.name == "leptos"))
    );

    let following = store
        .home_page(
            post::domain::home::HomeQuery {
                tab: post::domain::home::HomeTab::Following,
                ..Default::default()
            },
            None,
        )
        .expect("anonymous following home page");
    assert!(following.requires_login);
    assert!(following.topics.is_empty());
}

#[test]
fn home_page_ui_inventory_matches_sidebar_and_pagination_contract() {
    let text = post::app::home_seed_text();

    for required in [
        "首页",
        "帖子",
        "标签",
        "用户",
        "文档",
        "活动",
        "搜索帖子、标签、用户...",
        "显示 1-12 / 342 个主题",
        "分类",
        "热门标签",
        "公告",
        "活跃作者",
    ] {
        assert!(text.contains(required), "missing {required}");
    }

    assert!(!text.contains("系统功能"));
}

#[test]
fn api_routes_include_homepage_aggregate_endpoint() {
    let routes = post::app::api_route_inventory();

    assert!(routes.contains(&"/api/home"));
}

#[test]
fn api_routes_accept_app_state_runtime() {
    let state = post::state::AppState {
        db: None,
        forum: post::state::ForumStore::seeded(),
    };

    let _router = post::api::routes(state);
}

#[test]
fn search_contract_supports_keyword_highlight_filter_and_sort() {
    let store = post::state::ForumStore::seeded();

    let sqlx = store
        .search(post::domain::search::SearchQuery {
            q: "sqlx".to_string(),
            ..Default::default()
        })
        .expect("search sqlx");

    assert!(sqlx.total > 0);
    assert!(
        sqlx.items
            .iter()
            .any(|item| item.title_highlighted.contains("<mark>SQLx</mark>")
                || item.title_highlighted.contains("<mark>sqlx</mark>")
                || item.summary_highlighted.contains("<mark>SQLx</mark>")
                || item.summary_highlighted.contains("<mark>sqlx</mark>"))
    );

    let category = store
        .search(post::domain::search::SearchQuery {
            q: "leptos".to_string(),
            category: Some("经验分享".to_string()),
            ..Default::default()
        })
        .expect("category filtered search");
    assert!(
        category
            .items
            .iter()
            .all(|item| item.category_name.as_deref() == Some("经验分享"))
    );

    let tag = store
        .search(post::domain::search::SearchQuery {
            q: "leptos".to_string(),
            tag: Some("wasm".to_string()),
            ..Default::default()
        })
        .expect("tag filtered search");
    assert!(
        tag.items
            .iter()
            .all(|item| item.tags.iter().any(|tag| tag == "wasm"))
    );

    let hot = store
        .search(post::domain::search::SearchQuery {
            q: "leptos".to_string(),
            sort: post::domain::search::SearchSort::Hot,
            ..Default::default()
        })
        .expect("hot sorted search");
    assert!(
        hot.items
            .windows(2)
            .all(|pair| pair[0].score >= pair[1].score)
    );
}

#[test]
fn search_routes_are_registered() {
    assert!(post::app::primary_routes().contains(&"/search"));
    assert!(post::app::api_route_inventory().contains(&"/api/search"));
}

#[test]
fn notification_contract_records_business_events_and_read_state() {
    let store = post::state::ForumStore::seeded();
    let author = store.demo_user();
    let actor = store.login("alice", "password").expect("login actor").user;
    let post_id = store.list_posts().first().expect("seed post").post_id;

    store
        .add_comment(
            actor.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id,
                parent_comment_id: None,
                content: "这篇帖子很有帮助".to_string(),
            },
        )
        .expect("comment");

    let center = store
        .notification_center(author.user_id)
        .expect("author notifications");
    assert_eq!(center.unread_count, 1);
    assert_eq!(
        center.items[0].notification_type,
        post::domain::notifications::NotificationType::PostCommented
    );

    let notification_id = center.items[0].notification_id;
    store
        .mark_notification_read(author.user_id, notification_id)
        .expect("mark read");
    assert_eq!(
        store
            .notification_center(author.user_id)
            .expect("after read")
            .unread_count,
        0
    );

    store
        .toggle_post_like(actor.user_id, post_id)
        .expect("like post");
    store
        .toggle_post_like(actor.user_id, post_id)
        .expect("unlike post");
    let after_like = store
        .notification_center(author.user_id)
        .expect("like notifications");
    assert_eq!(
        after_like
            .items
            .iter()
            .filter(|item| item.notification_type
                == post::domain::notifications::NotificationType::PostLiked)
            .count(),
        1
    );

    store
        .mark_all_notifications_read(author.user_id)
        .expect("mark all read");
    assert_eq!(
        store
            .notification_center(author.user_id)
            .expect("after all read")
            .unread_count,
        0
    );
}

#[test]
fn notification_contract_notifies_followers_when_followed_user_posts() {
    let store = post::state::ForumStore::seeded();
    let author = store.demo_user();
    let follower = store.login("bob", "password").expect("login follower").user;

    store
        .follow_user(follower.user_id, author.user_id)
        .expect("follow author");
    store
        .create_post(
            author.user_id,
            post::domain::posts::CreatePostRequest {
                title: "Leptos 通知系统设计".to_string(),
                markdown: "发布后需要通知关注者".to_string(),
                summary: "关注推送".to_string(),
                category_name: Some("公告".to_string()),
                tag_names: vec!["leptos".to_string()],
                publish: true,
            },
        )
        .expect("publish post");

    let center = store
        .notification_center(follower.user_id)
        .expect("follower notifications");
    assert_eq!(center.unread_count, 1);
    assert_eq!(
        center.items[0].notification_type,
        post::domain::notifications::NotificationType::FollowedUserPosted
    );
}

#[test]
fn notification_routes_are_registered() {
    assert!(post::app::primary_routes().contains(&"/notifications"));
    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/notifications"));
    assert!(routes.contains(&"/api/notifications/{notification_id}/read"));
    assert!(routes.contains(&"/api/notifications/read-all"));
}

#[test]
fn notification_push_contract_tracks_online_connections_and_pending_payloads() {
    let store = post::state::ForumStore::seeded();
    let author = store.demo_user();
    let actor = store.login("push-actor", "password").expect("actor").user;
    let post_id = store.list_posts().first().expect("seed post").post_id;

    let connected = store
        .connect_notification_socket(author.user_id)
        .expect("connect notification socket");
    assert_eq!(connected.user_id, author.user_id);
    assert_eq!(connected.online_connections, 1);

    store
        .add_comment(
            actor.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id,
                parent_comment_id: None,
                content: "在线用户应该收到 WebSocket 推送".to_string(),
            },
        )
        .expect("comment creates notification");

    let pushes = store
        .pending_notification_pushes(author.user_id)
        .expect("pending pushes");
    assert_eq!(pushes.len(), 1);
    assert_eq!(pushes[0].recipient_id, author.user_id);
    assert_eq!(
        pushes[0].notification_type,
        post::domain::notifications::NotificationType::PostCommented
    );
    assert!(pushes[0].title.contains("push-actor"));
    assert!(pushes[0].body.contains("WebSocket"));

    store
        .ack_notification_push(author.user_id, pushes[0].push_id)
        .expect("ack push");
    assert!(
        store
            .pending_notification_pushes(author.user_id)
            .expect("pending after ack")
            .is_empty()
    );

    let disconnected = store
        .disconnect_notification_socket(author.user_id)
        .expect("disconnect notification socket");
    assert_eq!(disconnected.online_connections, 0);
}

#[test]
fn notification_push_routes_are_registered() {
    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/ws/notifications/{user_id}"));
    assert!(routes.contains(&"/api/notifications/online"));
    assert!(routes.contains(&"/api/notifications/pending-pushes"));
    assert!(routes.contains(&"/api/notifications/pending-pushes/{push_id}/ack"));
}

#[test]
fn notification_push_service_builds_payload_only_for_online_recipients() {
    let recipient_id = uuid::Uuid::from_u128(701);
    let notification = post::domain::notifications::Notification {
        notification_id: uuid::Uuid::from_u128(702),
        recipient_id,
        actor_id: Some(uuid::Uuid::from_u128(703)),
        notification_type: post::domain::notifications::NotificationType::PostCommented,
        title: "alice 评论了你的帖子".to_string(),
        body: "在线时需要生成 WebSocket 推送".to_string(),
        read_at: None,
        created_at: time::OffsetDateTime::now_utc(),
    };

    let push = post::services::notifications::NotificationPushService::build_pending_push(
        uuid::Uuid::from_u128(704),
        1,
        notification.clone(),
    )
    .expect("online recipient gets push");

    assert_eq!(push.push_id, uuid::Uuid::from_u128(704));
    assert_eq!(push.notification_id, notification.notification_id);
    assert_eq!(push.recipient_id, recipient_id);
    assert_eq!(push.notification_type, notification.notification_type);
    assert_eq!(push.title, notification.title);
    assert_eq!(push.body, notification.body);

    assert!(
        post::services::notifications::NotificationPushService::build_pending_push(
            uuid::Uuid::from_u128(705),
            0,
            notification,
        )
        .is_none()
    );
}

#[test]
fn admin_dashboard_requires_admin_and_exposes_rbac_menu() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let normal = store
        .login("member", "password")
        .expect("login member")
        .user;

    assert!(store.admin_dashboard(normal.user_id).is_err());

    let dashboard = store
        .admin_dashboard(admin.user_id)
        .expect("admin dashboard");
    assert!(dashboard.stats.iter().any(|stat| stat.label == "用户总数"));
    assert!(dashboard.stats.iter().any(|stat| stat.label == "帖子总数"));

    for required in [
        "用户管理",
        "角色管理",
        "权限管理",
        "帖子管理",
        "评论管理",
        "分类管理",
        "标签管理",
        "公告推送",
        "举报处理",
        "审计日志",
    ] {
        assert!(
            dashboard.menu.iter().any(|item| item.label == required),
            "missing admin menu {required}"
        );
    }

    assert!(
        dashboard
            .permissions
            .iter()
            .any(|permission| permission.code == "post:update")
    );
    assert!(
        dashboard
            .users
            .iter()
            .any(|user| user.actions.iter().any(|action| action == "禁用用户"))
    );
    assert!(!dashboard.moderation_posts.is_empty());
    assert!(dashboard.moderation_posts.iter().any(|post| {
        post.actions.iter().any(|action| action == "下架")
            && post.actions.iter().any(|action| action == "取消置顶")
    }));
    assert!(
        dashboard
            .moderation_comments
            .iter()
            .any(|comment| comment.actions.iter().any(|action| action == "恢复评论"))
    );
    assert!(
        dashboard
            .categories
            .iter()
            .any(|category| category.actions.iter().any(|action| action == "调整排序"))
    );
    assert!(
        dashboard
            .tags
            .iter()
            .any(|tag| tag.actions.iter().any(|action| action == "合并标签"))
    );
    assert!(dashboard.announcements.iter().any(|announcement| {
        announcement
            .actions
            .iter()
            .any(|action| action == "发布公告")
    }));
    assert!(
        dashboard
            .reports
            .iter()
            .any(|report| report.actions.iter().any(|action| action == "标记已处理"))
    );
    assert!(!dashboard.governance_queue.is_empty());
    assert!(dashboard.audit_entries.iter().all(|entry| {
        !entry.ip.is_empty() && !entry.user_agent.is_empty() && !entry.action.is_empty()
    }));
}

#[test]
fn admin_routes_are_registered() {
    assert!(post::app::primary_routes().contains(&"/admin"));
    assert!(post::app::api_route_inventory().contains(&"/api/admin/dashboard"));
}

#[test]
fn file_upload_contract_validates_image_metadata_and_markdown_url() {
    let store = post::state::ForumStore::seeded();
    let user = store.demo_user();

    let asset = store
        .upload_file(
            user.user_id,
            post::domain::files::FileUploadRequest {
                original_filename: "cover.png".to_string(),
                file_size: 128_000,
                mime_type: "image/png".to_string(),
                content_hash: "sha256-demo-cover".to_string(),
                usage: post::domain::files::FileUsage::MarkdownImage,
            },
        )
        .expect("valid image upload");

    assert_eq!(asset.bucket, "post-assets");
    assert!(asset.storage_key.ends_with("/cover.png"));
    assert_eq!(asset.public_url, format!("/uploads/{}", asset.storage_key));
    assert_eq!(
        asset.markdown_image,
        format!("![cover.png]({})", asset.public_url)
    );

    let invalid_type = store.upload_file(
        user.user_id,
        post::domain::files::FileUploadRequest {
            original_filename: "shell.svg".to_string(),
            file_size: 100,
            mime_type: "image/svg+xml".to_string(),
            content_hash: "sha256-svg".to_string(),
            usage: post::domain::files::FileUsage::MarkdownImage,
        },
    );
    assert!(invalid_type.is_err());

    let oversized = store.upload_file(
        user.user_id,
        post::domain::files::FileUploadRequest {
            original_filename: "huge.webp".to_string(),
            file_size: post::domain::files::MAX_IMAGE_SIZE_BYTES + 1,
            mime_type: "image/webp".to_string(),
            content_hash: "sha256-huge".to_string(),
            usage: post::domain::files::FileUsage::MarkdownImage,
        },
    );
    assert!(oversized.is_err());
}

#[test]
fn file_binary_upload_builds_server_side_hash_and_markdown_asset() {
    let store = post::state::ForumStore::seeded();
    let user = store.demo_user();
    let upload = post::domain::files::FileBinaryUploadRequest {
        original_filename: "cover binary.png".to_string(),
        mime_type: "image/png".to_string(),
        content_base64: "iVBORw0KGgo=".to_string(),
        usage: post::domain::files::FileUsage::MarkdownImage,
    };

    let metadata = upload.to_upload_request().expect("binary metadata");
    assert_eq!(metadata.original_filename, "cover binary.png");
    assert_eq!(metadata.file_size, 8);
    assert_eq!(metadata.mime_type, "image/png");
    assert_eq!(
        metadata.content_hash,
        "4c4b6a3be1314ab86138bef4314dde022e600960d8689a2c8f8631802d20dab6"
    );

    let asset = store
        .upload_binary_file(user.user_id, upload)
        .expect("binary image upload");
    assert_eq!(asset.file_hash, metadata.content_hash);
    assert!(asset.storage_key.ends_with("/cover-binary.png"));
    assert_eq!(
        asset.markdown_image,
        format!("![cover-binary.png]({})", asset.public_url)
    );

    let invalid_base64 = post::domain::files::FileBinaryUploadRequest {
        original_filename: "broken.png".to_string(),
        mime_type: "image/png".to_string(),
        content_base64: "not-base64".to_string(),
        usage: post::domain::files::FileUsage::MarkdownImage,
    };
    assert!(invalid_base64.to_upload_request().is_err());
}

#[test]
fn file_binary_upload_exposes_object_store_payload() {
    let upload = post::domain::files::FileBinaryUploadRequest {
        original_filename: "cover object.webp".to_string(),
        mime_type: "image/webp".to_string(),
        content_base64: "UklGRg==".to_string(),
        usage: post::domain::files::FileUsage::CoverImage,
    };

    let object = upload.to_object_upload().expect("object payload");
    assert_eq!(object.bytes, b"RIFF");
    assert_eq!(object.asset.file_size, 4);
    assert_eq!(object.asset.mime_type, "image/webp");
    assert_eq!(
        object.asset.content_hash,
        "a40ff3d5900fb7698b8c865041347cb49eccedc8f93945f89629ad104aaecce4"
    );
    assert_eq!(object.bucket, "post-assets");
    assert!(object.storage_key.ends_with("/cover-object.webp"));
    assert_eq!(object.content_type, "image/webp");
}

#[test]
fn rustfs_object_store_adapter_contract_uses_s3_put_object() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(manifest_dir.join("src/object_store.rs"))
        .expect("RustFS object store adapter should exist");

    for required in [
        "aws_credential_types::Credentials",
        "aws_sdk_s3::{Client, config::Region, primitives::ByteStream}",
        "Credentials::new",
        "aws_sdk_s3::config::Builder::new()",
        ".behavior_version_latest()",
        ".region(Region::new(config.region))",
        ".credentials_provider(credentials)",
        ".endpoint_url(config.endpoint_url)",
        ".force_path_style(config.force_path_style)",
        "Client::from_conf(s3_config)",
        "async fn ensure_bucket",
        ".head_bucket()",
        ".create_bucket()",
        "self.ensure_bucket().await?",
        ".put_object()",
        ".bucket(self.bucket.as_str())",
        ".key(object.storage_key)",
        ".content_type(object.content_type)",
        ".body(ByteStream::from(object.bytes))",
        ".send()",
    ] {
        assert!(
            source.contains(required),
            "missing RustFS adapter fragment: {required}"
        );
    }

    let state = include_str!("../src/state.rs");
    assert!(state.contains("RustfsObjectStore::from_config"));
    assert!(state.contains(".put_object(object.clone())"));

    let env = include_str!("../.env.example");
    for required in [
        "RUSTFS_BUCKET=post-assets",
        "RUSTFS_REGION=us-east-1",
        "RUSTFS_ACCESS_KEY=rustfsadmin",
        "RUSTFS_SECRET_KEY=rustfsadmin",
        "RUSTFS_FORCE_PATH_STYLE=true",
    ] {
        assert!(
            env.contains(required),
            "missing .env.example entry: {required}"
        );
    }

    let compose = include_str!("../docker-compose.yml");
    for required in [
        "RUSTFS_ACCESS_KEY: rustfsadmin",
        "RUSTFS_SECRET_KEY: rustfsadmin",
        "RUSTFS_CONSOLE_ENABLE: \"true\"",
    ] {
        assert!(
            compose.contains(required),
            "missing docker compose entry: {required}"
        );
    }
}

#[test]
fn binary_upload_checks_postgres_hash_before_writing_rustfs_object() {
    let state = include_str!("../src/state.rs");
    let start = state
        .find("pub async fn upload_binary_file")
        .expect("upload_binary_file should exist");
    let end = state[start..]
        .find("pub async fn admin_dashboard")
        .expect("upload_binary_file should end before admin_dashboard")
        + start;
    let upload_binary = &state[start..end];

    let dedupe_index = upload_binary
        .find("PostgresFileRepository::find_by_hash(pool, &object.asset.content_hash)")
        .expect("binary uploads should check existing file hash before writing object storage");
    let put_index = upload_binary
        .find(".put_object(object.clone())")
        .expect("binary uploads should write the object for new hashes");

    assert!(
        dedupe_index < put_index,
        "binary uploads must return existing metadata before writing RustFS to avoid orphan objects"
    );
}

#[test]
fn file_upload_routes_and_editor_inventory_are_registered() {
    assert!(post::app::api_route_inventory().contains(&"/api/files"));
    assert!(post::app::api_route_inventory().contains(&"/api/files/binary"));
    let text = post::app::ui_feature_inventory();
    for required in ["图片上传", "MIME 类型", "文件大小", "Markdown 图片链接"] {
        assert!(text.contains(required), "missing {required}");
    }

    let api = include_str!("../src/api.rs");
    assert!(api.contains("upload_binary_file"));
}

#[test]
fn user_space_contract_aggregates_profile_activity_and_relationships() {
    let store = post::state::ForumStore::seeded();
    let author = store.demo_user();
    let viewer = store
        .login("reader", "password")
        .expect("login viewer")
        .user;
    let post = store.list_posts().first().expect("seed post").clone();

    store
        .toggle_post_favorite(viewer.user_id, post.post_id)
        .expect("favorite post");
    store
        .follow_user(viewer.user_id, author.user_id)
        .expect("follow author");
    store
        .add_comment(
            viewer.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: post.post_id,
                parent_comment_id: None,
                content: "收藏备用".to_string(),
            },
        )
        .expect("comment");

    let author_space = store
        .user_space(author.user_id, Some(viewer.user_id))
        .expect("author space");
    assert_eq!(author_space.profile.user_id, author.user_id);
    assert!(author_space.stats.published_posts >= 1);
    assert!(author_space.stats.followers >= 1);
    assert!(author_space.followed_by_viewer);
    assert!(!author_space.published_posts.is_empty());

    let viewer_space = store
        .user_space(viewer.user_id, Some(viewer.user_id))
        .expect("viewer space");
    assert_eq!(viewer_space.favorite_posts.len(), 1);
    assert_eq!(viewer_space.following.len(), 1);
    assert_eq!(viewer_space.comments.len(), 1);
    assert!(viewer_space.is_me);
}

#[test]
fn user_space_routes_are_registered() {
    let routes = post::app::primary_routes();
    assert!(routes.contains(&"/users/sample"));
    assert!(routes.contains(&"/me"));
    assert!(post::app::api_route_inventory().contains(&"/api/users/{user_id}/space"));
}

#[test]
fn user_profile_contract_supports_profile_avatar_and_password_updates() {
    let store = post::state::ForumStore::seeded();
    let user = store
        .login("profile-user", "password")
        .expect("login profile user")
        .user;

    let profile = store
        .update_profile(
            user.user_id,
            post::domain::users::UpdateProfileRequest {
                nickname: "Profile User".to_string(),
                bio: "专注 Leptos、Axum 和 SQLx 的全栈开发者。".to_string(),
            },
        )
        .expect("update profile");
    assert_eq!(profile.nickname, "Profile User");
    assert_eq!(profile.bio, "专注 Leptos、Axum 和 SQLx 的全栈开发者。");

    let avatar = store
        .update_avatar(
            user.user_id,
            post::domain::users::UpdateAvatarRequest {
                avatar_url: "/uploads/avatars/profile-user.png".to_string(),
            },
        )
        .expect("update avatar");
    assert_eq!(
        avatar.avatar_url.as_deref(),
        Some("/uploads/avatars/profile-user.png")
    );

    let space = store
        .user_space(user.user_id, Some(user.user_id))
        .expect("user space");
    assert_eq!(space.profile.nickname, "Profile User");
    assert_eq!(space.profile.avatar_url, avatar.avatar_url);
    assert!(space.is_me);

    assert!(
        store
            .change_password(
                user.user_id,
                post::domain::users::ChangePasswordRequest {
                    old_password: "wrong-password".to_string(),
                    new_password: "new-password".to_string(),
                },
            )
            .is_err()
    );
    store
        .change_password(
            user.user_id,
            post::domain::users::ChangePasswordRequest {
                old_password: "password".to_string(),
                new_password: "new-password".to_string(),
            },
        )
        .expect("change password");
    assert!(store.login("profile-user", "password").is_err());
    assert!(store.login("profile-user", "new-password").is_ok());
}

#[test]
fn user_settings_service_normalizes_and_validates_settings() {
    let blank_profile = post::services::users::UserSettingsService::normalize_profile(
        post::domain::users::UpdateProfileRequest {
            nickname: "   ".to_string(),
            bio: "简介".to_string(),
        },
    );
    assert!(blank_profile.is_err());

    let long_bio = post::services::users::UserSettingsService::normalize_profile(
        post::domain::users::UpdateProfileRequest {
            nickname: "用户".to_string(),
            bio: "a".repeat(161),
        },
    );
    assert!(long_bio.is_err());

    let profile = post::services::users::UserSettingsService::normalize_profile(
        post::domain::users::UpdateProfileRequest {
            nickname: "  用户昵称  ".to_string(),
            bio: "  个人简介  ".to_string(),
        },
    )
    .expect("profile");
    assert_eq!(profile.nickname, "用户昵称");
    assert_eq!(profile.bio, "个人简介");

    assert!(
        post::services::users::UserSettingsService::normalize_avatar(
            post::domain::users::UpdateAvatarRequest {
                avatar_url: "ftp://avatar.png".to_string(),
            },
        )
        .is_err()
    );
    assert_eq!(
        post::services::users::UserSettingsService::normalize_avatar(
            post::domain::users::UpdateAvatarRequest {
                avatar_url: "  /uploads/avatar.png  ".to_string(),
            },
        )
        .expect("avatar"),
        "/uploads/avatar.png"
    );

    assert!(
        post::services::users::UserSettingsService::validate_password_change(
            "password",
            post::domain::users::ChangePasswordRequest {
                old_password: "wrong".to_string(),
                new_password: "new-password".to_string(),
            },
        )
        .is_err()
    );
    assert!(
        post::services::users::UserSettingsService::validate_password_change(
            "password",
            post::domain::users::ChangePasswordRequest {
                old_password: "password".to_string(),
                new_password: "123".to_string(),
            },
        )
        .is_err()
    );
    assert_eq!(
        post::services::users::UserSettingsService::validate_password_change(
            "password",
            post::domain::users::ChangePasswordRequest {
                old_password: " password ".to_string(),
                new_password: " new-password ".to_string(),
            },
        )
        .expect("password"),
        "new-password"
    );
}

#[test]
fn user_profile_routes_are_registered() {
    let primary = post::app::primary_routes();
    for route in [
        "/me/posts",
        "/me/drafts",
        "/me/comments",
        "/me/favorites",
        "/me/following",
        "/me/followers",
    ] {
        assert!(primary.contains(&route), "missing primary route {route}");
    }

    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/users/{user_id}/profile"));
    assert!(routes.contains(&"/api/users/{user_id}/avatar"));
    assert!(routes.contains(&"/api/users/{user_id}/password"));
}

#[test]
fn report_contract_supports_submission_and_admin_resolution() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let reporter = store
        .login("reporter", "password")
        .expect("login reporter")
        .user;
    let normal = store
        .login("normal", "password")
        .expect("login normal")
        .user;
    let post_id = store.list_posts().first().expect("seed post").post_id;

    let report = store
        .create_report(
            reporter.user_id,
            post::domain::reports::CreateReportRequest {
                target_type: post::domain::reports::ReportTargetType::Post,
                target_id: post_id,
                reason: "垃圾广告".to_string(),
                description: Some("正文里包含明显推广链接".to_string()),
            },
        )
        .expect("create report");

    assert_eq!(report.reporter_id, reporter.user_id);
    assert_eq!(report.status, post::domain::reports::ReportStatus::Pending);
    assert_eq!(
        report.target_title.as_deref(),
        Some("Rust 异步任务的边界设计")
    );

    assert!(store.list_reports(normal.user_id).is_err());

    let pending = store.list_reports(admin.user_id).expect("admin reports");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].reason, "垃圾广告");

    let handled = store
        .handle_report(
            admin.user_id,
            report.report_id,
            post::domain::reports::HandleReportRequest {
                status: post::domain::reports::ReportStatus::Handled,
                note: "已下架违规内容".to_string(),
            },
        )
        .expect("handle report");
    assert_eq!(handled.status, post::domain::reports::ReportStatus::Handled);
    assert_eq!(handled.handler_id, Some(admin.user_id));
    assert_eq!(
        handled.handler_name.as_deref(),
        Some(admin.nickname.as_str())
    );
    assert!(handled.handled_at.is_some());
    assert_eq!(handled.handle_note.as_deref(), Some("已下架违规内容"));
}

#[test]
fn report_service_builds_and_handles_reports() {
    let report_id = uuid::Uuid::from_u128(1201);
    let reporter_id = uuid::Uuid::from_u128(1202);
    let handler_id = uuid::Uuid::from_u128(1203);
    let target_id = uuid::Uuid::from_u128(1204);
    let created_at = time::OffsetDateTime::now_utc();
    let handled_at = created_at + time::Duration::minutes(5);

    let invalid = post::services::reports::ReportService::build_report(
        report_id,
        reporter_id,
        "Reporter",
        Some("目标标题".to_string()),
        post::domain::reports::CreateReportRequest {
            target_type: post::domain::reports::ReportTargetType::Post,
            target_id,
            reason: "   ".to_string(),
            description: None,
        },
        created_at,
    );
    assert!(invalid.is_err());

    let mut report = post::services::reports::ReportService::build_report(
        report_id,
        reporter_id,
        "Reporter",
        Some("目标标题".to_string()),
        post::domain::reports::CreateReportRequest {
            target_type: post::domain::reports::ReportTargetType::Post,
            target_id,
            reason: "  垃圾广告  ".to_string(),
            description: Some("   ".to_string()),
        },
        created_at,
    )
    .expect("build report");

    assert_eq!(report.reason, "垃圾广告");
    assert!(report.description.is_none());
    assert_eq!(report.status, post::domain::reports::ReportStatus::Pending);
    assert_eq!(report.created_at, created_at);

    post::services::reports::ReportService::apply_handle(
        &mut report,
        handler_id,
        "Admin",
        post::domain::reports::HandleReportRequest {
            status: post::domain::reports::ReportStatus::Rejected,
            note: "  证据不足  ".to_string(),
        },
        handled_at,
    )
    .expect("handle report");

    assert_eq!(report.status, post::domain::reports::ReportStatus::Rejected);
    assert_eq!(report.handler_id, Some(handler_id));
    assert_eq!(report.handler_name.as_deref(), Some("Admin"));
    assert_eq!(report.handle_note.as_deref(), Some("证据不足"));
    assert_eq!(report.handled_at, Some(handled_at));

    let pending = post::services::reports::ReportService::apply_handle(
        &mut report,
        handler_id,
        "Admin",
        post::domain::reports::HandleReportRequest {
            status: post::domain::reports::ReportStatus::Pending,
            note: "回退".to_string(),
        },
        handled_at,
    );
    assert!(pending.is_err());
}

#[test]
fn report_routes_are_registered() {
    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/reports"));
    assert!(routes.contains(&"/api/admin/reports"));
    assert!(routes.contains(&"/api/admin/reports/{report_id}/handle"));
}

#[test]
fn announcement_contract_supports_publish_push_withdraw_and_read_state() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let member = store
        .login("reader", "password")
        .expect("login member")
        .user;

    let normal_create = store.create_announcement(
        member.user_id,
        post::domain::announcements::CreateAnnouncementRequest {
            title: "普通用户不能发布".to_string(),
            content: "权限不足".to_string(),
            announcement_type: "system".to_string(),
            pinned: false,
            effective_at: None,
            expires_at: None,
            audience: post::domain::announcements::AnnouncementAudience::AllUsers,
        },
    );
    assert!(normal_create.is_err());

    let draft = store
        .create_announcement(
            admin.user_id,
            post::domain::announcements::CreateAnnouncementRequest {
                title: "论坛维护通知".to_string(),
                content: "今晚 22:00 将进行搜索索引升级。".to_string(),
                announcement_type: "maintenance".to_string(),
                pinned: true,
                effective_at: None,
                expires_at: None,
                audience: post::domain::announcements::AnnouncementAudience::AllUsers,
            },
        )
        .expect("create announcement");
    assert_eq!(
        draft.status,
        post::domain::announcements::AnnouncementStatus::Draft
    );

    let published = store
        .publish_announcement(admin.user_id, draft.announcement_id)
        .expect("publish announcement");
    assert_eq!(
        published.status,
        post::domain::announcements::AnnouncementStatus::Published
    );
    assert!(published.published_at.is_some());

    let member_center = store
        .notification_center(member.user_id)
        .expect("member notifications");
    assert!(member_center.items.iter().any(|item| item.notification_type
        == post::domain::notifications::NotificationType::Announcement
        && item.title == "论坛维护通知"));

    let public_announcements = store.public_announcements();
    assert!(
        public_announcements
            .iter()
            .any(|item| item.title == "论坛维护通知")
    );

    let home = store
        .home_page(
            post::domain::home::HomeQuery::default(),
            Some(member.user_id),
        )
        .expect("home with announcement");
    assert!(
        home.announcements
            .iter()
            .any(|announcement| announcement.title == "论坛维护通知")
    );

    let read_state = store
        .mark_announcement_read(member.user_id, draft.announcement_id)
        .expect("mark announcement read");
    assert!(read_state.read);

    let withdrawn = store
        .withdraw_announcement(admin.user_id, draft.announcement_id)
        .expect("withdraw announcement");
    assert_eq!(
        withdrawn.status,
        post::domain::announcements::AnnouncementStatus::Withdrawn
    );
    assert!(
        store
            .public_announcements()
            .iter()
            .all(|item| item.title != "论坛维护通知")
    );
}

#[test]
fn announcement_service_builds_and_transitions_announcements() {
    let announcement_id = uuid::Uuid::from_u128(1101);
    let creator_id = uuid::Uuid::from_u128(1102);
    let now = time::OffsetDateTime::now_utc();
    let request = post::domain::announcements::CreateAnnouncementRequest {
        title: "  论坛维护通知  ".to_string(),
        content: "  今晚 22:00 将进行搜索索引升级。  ".to_string(),
        announcement_type: "  maintenance  ".to_string(),
        pinned: true,
        effective_at: Some(now),
        expires_at: Some(now + time::Duration::days(1)),
        audience: post::domain::announcements::AnnouncementAudience::AllUsers,
    };

    let mut announcement = post::services::announcements::AnnouncementService::build_draft(
        announcement_id,
        creator_id,
        "管理员",
        request,
        now,
    )
    .expect("build draft announcement");

    assert_eq!(announcement.announcement_id, announcement_id);
    assert_eq!(announcement.title, "论坛维护通知");
    assert_eq!(announcement.content, "今晚 22:00 将进行搜索索引升级。");
    assert_eq!(announcement.announcement_type, "maintenance");
    assert_eq!(
        announcement.status,
        post::domain::announcements::AnnouncementStatus::Draft
    );
    assert!(announcement.published_at.is_none());

    post::services::announcements::AnnouncementService::publish(&mut announcement, now)
        .expect("publish announcement");
    assert_eq!(
        announcement.status,
        post::domain::announcements::AnnouncementStatus::Published
    );
    assert_eq!(announcement.published_at, Some(now));
    assert!(announcement.withdrawn_at.is_none());

    let long_body = "公告内容".repeat(40);
    let body = post::services::announcements::AnnouncementService::notification_body(&long_body);
    assert_eq!(body.chars().count(), 120);

    post::services::announcements::AnnouncementService::withdraw(&mut announcement, now)
        .expect("withdraw announcement");
    assert_eq!(
        announcement.status,
        post::domain::announcements::AnnouncementStatus::Withdrawn
    );
    assert_eq!(announcement.withdrawn_at, Some(now));
}

#[test]
fn announcement_routes_are_registered() {
    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/announcements"));
    assert!(routes.contains(&"/api/announcements/{announcement_id}/read"));
    assert!(routes.contains(&"/api/admin/announcements"));
    assert!(routes.contains(&"/api/admin/announcements/{announcement_id}/publish"));
    assert!(routes.contains(&"/api/admin/announcements/{announcement_id}/withdraw"));
}

#[test]
fn taxonomy_contract_supports_admin_category_and_tag_management() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let member = store
        .login("taxonomy-member", "password")
        .expect("login member")
        .user;

    let categories = store.public_categories();
    assert_eq!(categories.len(), 6);
    assert!(
        categories
            .iter()
            .any(|category| category.name == "公告" && category.post_count == 12)
    );

    let normal_create = store.create_category(
        member.user_id,
        post::domain::taxonomy::CreateCategoryRequest {
            name: "源码解析".to_string(),
            color: "#0064E0".to_string(),
            sort_order: 10,
        },
    );
    assert!(normal_create.is_err());

    let category = store
        .create_category(
            admin.user_id,
            post::domain::taxonomy::CreateCategoryRequest {
                name: "源码解析".to_string(),
                color: "#0064E0".to_string(),
                sort_order: 10,
            },
        )
        .expect("create category");
    let updated_category = store
        .update_category(
            admin.user_id,
            category.category_id,
            post::domain::taxonomy::UpdateCategoryRequest {
                name: Some("源码阅读".to_string()),
                color: Some("#0A7CFF".to_string()),
                sort_order: Some(2),
                enabled: Some(true),
            },
        )
        .expect("update category");
    assert_eq!(updated_category.name, "源码阅读");
    assert_eq!(updated_category.sort_order, 2);

    store
        .disable_category(admin.user_id, category.category_id)
        .expect("disable category");
    assert!(
        store
            .public_categories()
            .iter()
            .all(|item| item.name != "源码阅读")
    );

    let leptos = store
        .public_tags()
        .into_iter()
        .find(|tag| tag.name == "leptos")
        .expect("leptos tag");
    let source = store
        .create_tag(
            admin.user_id,
            post::domain::taxonomy::CreateTagRequest {
                name: "leptos-ssr".to_string(),
                sort_order: 1,
            },
        )
        .expect("create tag");
    let source = store
        .update_tag(
            admin.user_id,
            source.tag_id,
            post::domain::taxonomy::UpdateTagRequest {
                name: Some("leptos-ssr".to_string()),
                sort_order: Some(1),
                enabled: Some(true),
                use_count: Some(7),
            },
        )
        .expect("update tag");
    let merged = store
        .merge_tag(
            admin.user_id,
            source.tag_id,
            post::domain::taxonomy::MergeTagRequest {
                target_tag_id: leptos.tag_id,
            },
        )
        .expect("merge tag");
    assert_eq!(merged.use_count, leptos.use_count + source.use_count);
    assert!(
        store
            .public_tags()
            .iter()
            .all(|tag| tag.name != "leptos-ssr")
    );

    let obsolete = store
        .create_tag(
            admin.user_id,
            post::domain::taxonomy::CreateTagRequest {
                name: "obsolete".to_string(),
                sort_order: 99,
            },
        )
        .expect("create obsolete tag");
    store
        .delete_tag(admin.user_id, obsolete.tag_id)
        .expect("delete tag");
    assert!(store.public_tags().iter().all(|tag| tag.name != "obsolete"));

    let home = store
        .home_page(post::domain::home::HomeQuery::default(), None)
        .expect("home page");
    assert!(home.categories.iter().all(|item| item.name != "源码阅读"));
    assert!(
        home.hot_tags
            .iter()
            .any(|tag| tag.name == "leptos" && tag.count == 139)
    );
}

#[test]
fn taxonomy_service_normalizes_and_merges_categories_and_tags() {
    let category_id = uuid::Uuid::from_u128(1001);
    let mut category = post::services::taxonomy::TaxonomyService::build_category(
        category_id,
        post::domain::taxonomy::CreateCategoryRequest {
            name: "  源码解析  ".to_string(),
            color: " #0064E0 ".to_string(),
            sort_order: 8,
        },
    )
    .expect("build category");
    assert_eq!(category.category_id, category_id);
    assert_eq!(category.name, "源码解析");
    assert_eq!(category.color, "#0064E0");
    assert_eq!(category.sort_order, 8);
    assert!(category.enabled);
    assert_eq!(category.post_count, 0);

    post::services::taxonomy::TaxonomyService::apply_category_update(
        &mut category,
        post::domain::taxonomy::UpdateCategoryRequest {
            name: Some("  源码阅读  ".to_string()),
            color: Some(" #0A7CFF ".to_string()),
            sort_order: Some(2),
            enabled: Some(false),
        },
    )
    .expect("update category");
    assert_eq!(category.name, "源码阅读");
    assert_eq!(category.color, "#0A7CFF");
    assert_eq!(category.sort_order, 2);
    assert!(!category.enabled);

    let source_id = uuid::Uuid::from_u128(1002);
    let target_id = uuid::Uuid::from_u128(1003);
    let mut source = post::services::taxonomy::TaxonomyService::build_tag(
        source_id,
        post::domain::taxonomy::CreateTagRequest {
            name: "  Leptos-SSR  ".to_string(),
            sort_order: 4,
        },
    )
    .expect("build source tag");
    assert_eq!(source.name, "leptos-ssr");

    post::services::taxonomy::TaxonomyService::apply_tag_update(
        &mut source,
        post::domain::taxonomy::UpdateTagRequest {
            name: Some("  Leptos-SSR  ".to_string()),
            sort_order: Some(5),
            enabled: Some(true),
            use_count: Some(7),
        },
    )
    .expect("update source tag");
    assert_eq!(source.name, "leptos-ssr");
    assert_eq!(source.sort_order, 5);
    assert_eq!(source.use_count, 7);

    let mut target = post::domain::taxonomy::TagItem {
        tag_id: target_id,
        name: "leptos".to_string(),
        sort_order: 1,
        enabled: true,
        use_count: 132,
    };
    assert!(
        post::services::taxonomy::TaxonomyService::validate_tag_merge(source_id, source_id)
            .is_err()
    );
    post::services::taxonomy::TaxonomyService::validate_tag_merge(source_id, target_id)
        .expect("valid merge");
    post::services::taxonomy::TaxonomyService::apply_target_merge(&mut target, source.use_count);
    post::services::taxonomy::TaxonomyService::disable_merged_source(&mut source);
    assert_eq!(target.use_count, 139);
    assert!(target.enabled);
    assert_eq!(source.use_count, 0);
    assert!(!source.enabled);
}

#[test]
fn taxonomy_routes_are_registered() {
    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/categories"));
    assert!(routes.contains(&"/api/tags"));
    assert!(routes.contains(&"/api/admin/categories"));
    assert!(routes.contains(&"/api/admin/categories/{category_id}/update"));
    assert!(routes.contains(&"/api/admin/categories/{category_id}/disable"));
    assert!(routes.contains(&"/api/admin/tags"));
    assert!(routes.contains(&"/api/admin/tags/{tag_id}/update"));
    assert!(routes.contains(&"/api/admin/tags/{tag_id}/merge"));
    assert!(routes.contains(&"/api/admin/tags/{tag_id}/delete"));
}

#[test]
fn content_moderation_contract_supports_post_and_comment_actions() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let member = store
        .login("content-member", "password")
        .expect("login member")
        .user;
    let post_id = store.list_posts().first().expect("seed post").post_id;

    assert!(store.admin_posts(member.user_id).is_err());
    assert!(store.admin_posts(admin.user_id).expect("admin posts").len() >= 1);

    let offline = store
        .take_down_post(admin.user_id, post_id)
        .expect("take down post");
    assert_eq!(offline.status, post::domain::posts::PostStatus::Offline);
    assert!(
        store
            .list_posts()
            .iter()
            .all(|post| post.post_id != post_id)
    );

    let restored = store
        .restore_post(admin.user_id, post_id)
        .expect("restore post");
    assert_eq!(restored.status, post::domain::posts::PostStatus::Published);
    assert!(
        store
            .list_posts()
            .iter()
            .any(|post| post.post_id == post_id)
    );

    let pinned = store.pin_post(admin.user_id, post_id).expect("pin post");
    assert!(pinned.pinned);
    let unpinned = store
        .unpin_post(admin.user_id, post_id)
        .expect("unpin post");
    assert!(!unpinned.pinned);

    let comment = store
        .add_comment(
            member.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id,
                parent_comment_id: None,
                content: "这条评论需要审核".to_string(),
            },
        )
        .expect("comment");
    assert_eq!(store.post_detail(post_id).unwrap().summary.comment_count, 1);

    let deleted_comment = store
        .delete_comment(admin.user_id, comment.comment_id)
        .expect("delete comment");
    assert!(deleted_comment.deleted);
    assert_eq!(store.post_detail(post_id).unwrap().summary.comment_count, 0);
    let public_comments = store.comments_for_post(post_id).expect("comments");
    assert_eq!(public_comments[0].content, "该评论已被删除");

    let recovered_comment = store
        .recover_comment(admin.user_id, comment.comment_id)
        .expect("recover comment");
    assert!(!recovered_comment.deleted);
    assert_eq!(store.post_detail(post_id).unwrap().summary.comment_count, 1);

    let deleted_post = store
        .delete_post(admin.user_id, post_id)
        .expect("delete post");
    assert_eq!(
        deleted_post.status,
        post::domain::posts::PostStatus::Deleted
    );
    assert!(
        store
            .list_posts()
            .iter()
            .all(|post| post.post_id != post_id)
    );
}

#[test]
fn moderation_service_applies_post_and_comment_actions() {
    let post_id = uuid::Uuid::from_u128(1201);
    let author_id = uuid::Uuid::from_u128(1202);
    let now = time::OffsetDateTime::now_utc();
    let mut detail = post::domain::posts::PostDetail {
        summary: post::domain::posts::PostSummary {
            post_id,
            title: "待审核帖子".to_string(),
            summary: "帖子摘要".to_string(),
            author_id,
            author_name: "作者".to_string(),
            author_avatar_url: None,
            category_name: Some("教程".to_string()),
            tags: vec!["rust".to_string()],
            view_count: 42,
            comment_count: 2,
            like_count: 0,
            favorite_count: 0,
            published_at: Some(now),
        },
        markdown: "正文".to_string(),
        sanitized_html: "<p>正文</p>".to_string(),
        status: post::domain::posts::PostStatus::Published,
        liked_by_me: false,
        favorited_by_me: false,
        following_author: false,
    };

    let offline = post::services::moderation::ModerationService::apply_post_status(
        &mut detail,
        post::domain::posts::PostStatus::Offline,
        true,
    );
    assert_eq!(offline.status, post::domain::posts::PostStatus::Offline);
    assert!(offline.pinned);
    assert_eq!(detail.status, post::domain::posts::PostStatus::Offline);

    let deleted = post::services::moderation::ModerationService::apply_post_status(
        &mut detail,
        post::domain::posts::PostStatus::Deleted,
        true,
    );
    assert_eq!(deleted.status, post::domain::posts::PostStatus::Deleted);
    assert!(!deleted.pinned);
    assert!(
        post::services::moderation::ModerationService::build_pin_action(&detail, true).is_err()
    );

    detail.status = post::domain::posts::PostStatus::Published;
    let pinned = post::services::moderation::ModerationService::build_pin_action(&detail, true)
        .expect("pin published post");
    assert!(pinned.pinned);

    let comment_id = uuid::Uuid::from_u128(1203);
    let mut comment = post::domain::comments::CommentNode {
        comment_id,
        post_id,
        parent_comment_id: None,
        author_id,
        author_name: "作者".to_string(),
        content: "评论内容".to_string(),
        deleted: false,
        author_reply: false,
        like_count: 0,
        created_at: now,
        replies: vec![post::domain::comments::CommentNode {
            comment_id: uuid::Uuid::from_u128(1204),
            post_id,
            parent_comment_id: Some(comment_id),
            author_id,
            author_name: "作者".to_string(),
            content: "回复内容".to_string(),
            deleted: false,
            author_reply: false,
            like_count: 0,
            created_at: now,
            replies: Vec::new(),
        }],
    };
    let delete_effect =
        post::services::moderation::ModerationService::apply_comment_deleted(&mut comment, true);
    assert!(delete_effect.action.deleted);
    assert_eq!(delete_effect.count_delta, -1);
    post::services::moderation::ModerationService::apply_comment_count_delta(
        &mut detail,
        delete_effect.count_delta,
    );
    assert_eq!(detail.summary.comment_count, 1);

    let recover_effect =
        post::services::moderation::ModerationService::apply_comment_deleted(&mut comment, false);
    assert!(!recover_effect.action.deleted);
    assert_eq!(recover_effect.count_delta, 1);

    let row = post::services::moderation::ModerationService::post_row(&detail, true);
    assert_eq!(row.post_id, post_id);
    assert!(row.pinned);

    let rows = post::services::moderation::ModerationService::flatten_comment_rows(
        post_id,
        "待审核帖子",
        &[comment],
    );
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].post_title, "待审核帖子");
}

#[test]
fn comment_interaction_contract_supports_author_delete_like_and_report() {
    let store = post::state::ForumStore::seeded();
    let post_author = store.demo_user();
    let commenter = store
        .login("comment-author", "password")
        .expect("comment author")
        .user;
    let viewer = store
        .login("comment-viewer", "password")
        .expect("comment viewer")
        .user;
    let post_id = store.list_posts().first().expect("seed post").post_id;

    let comment = store
        .add_comment(
            commenter.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id,
                parent_comment_id: None,
                content: "这是一条可以互动的评论".to_string(),
            },
        )
        .expect("create comment");
    assert_eq!(store.post_detail(post_id).unwrap().summary.comment_count, 1);

    let liked = store
        .toggle_comment_like(viewer.user_id, comment.comment_id)
        .expect("like comment");
    assert!(liked.active);
    assert_eq!(liked.count, 1);
    let commenter_center = store
        .notification_center(commenter.user_id)
        .expect("commenter notifications");
    assert!(commenter_center.items.iter().any(|item| {
        item.notification_type == post::domain::notifications::NotificationType::CommentLiked
            && item.title.contains("点赞")
    }));

    let unliked = store
        .toggle_comment_like(viewer.user_id, comment.comment_id)
        .expect("unlike comment");
    assert!(!unliked.active);
    assert_eq!(unliked.count, 0);

    let report = store
        .report_comment(
            viewer.user_id,
            comment.comment_id,
            post::domain::reports::CreateReportRequest {
                target_type: post::domain::reports::ReportTargetType::Comment,
                target_id: comment.comment_id,
                reason: "人身攻击".to_string(),
                description: Some("评论内容不友好".to_string()),
            },
        )
        .expect("report comment");
    assert_eq!(
        report.target_type,
        post::domain::reports::ReportTargetType::Comment
    );
    assert_eq!(report.target_id, comment.comment_id);
    assert_eq!(report.reporter_id, viewer.user_id);
    assert_eq!(report.status, post::domain::reports::ReportStatus::Pending);
    assert_eq!(
        report.target_title.as_deref(),
        Some("这是一条可以互动的评论")
    );

    assert!(
        store
            .delete_own_comment(viewer.user_id, comment.comment_id)
            .is_err()
    );
    let deleted = store
        .delete_own_comment(commenter.user_id, comment.comment_id)
        .expect("delete own comment");
    assert!(deleted.deleted);
    assert_eq!(deleted.content, "该评论已被删除");
    assert_eq!(store.post_detail(post_id).unwrap().summary.comment_count, 0);
    let public_comments = store.comments_for_post(post_id).expect("public comments");
    assert_eq!(public_comments[0].content, "该评论已被删除");

    let post_author_center = store
        .notification_center(post_author.user_id)
        .expect("post author notifications");
    assert!(
        post_author_center
            .items
            .iter()
            .any(|item| item.notification_type
                == post::domain::notifications::NotificationType::PostCommented)
    );
}

#[test]
fn comment_service_builds_masks_and_summarizes_comments() {
    let post_id = uuid::Uuid::from_u128(901);
    let post_author_id = uuid::Uuid::from_u128(902);
    let commenter_id = post_author_id;
    let comment_id = uuid::Uuid::from_u128(903);
    let now = time::OffsetDateTime::now_utc();

    let invalid = post::services::comments::CommentService::build_comment(
        comment_id,
        post_id,
        None,
        commenter_id,
        "作者",
        post_author_id,
        "   ",
        now,
    );
    assert!(invalid.is_err());

    let mut comment = post::services::comments::CommentService::build_comment(
        comment_id,
        post_id,
        None,
        commenter_id,
        "作者",
        post_author_id,
        "  这是一条作者回复  ",
        now,
    )
    .expect("build comment");
    assert_eq!(comment.content, "这是一条作者回复");
    assert!(comment.author_reply);
    assert_eq!(comment.like_count, 0);
    assert_eq!(comment.created_at, now);

    comment.replies.push(
        post::services::comments::CommentService::build_comment(
            uuid::Uuid::from_u128(904),
            post_id,
            Some(comment_id),
            uuid::Uuid::from_u128(905),
            "读者",
            post_author_id,
            "这是一条需要隐藏的回复",
            now,
        )
        .expect("build reply"),
    );
    comment.deleted = true;
    comment.replies[0].deleted = true;

    let masked = post::services::comments::CommentService::mask_deleted(comment);
    assert_eq!(masked.content, "该评论已被删除");
    assert_eq!(masked.replies[0].content, "该评论已被删除");

    let long_content = "a".repeat(120);
    assert_eq!(
        post::services::comments::CommentService::notification_body(&long_content).len(),
        80
    );
}

#[test]
fn reaction_service_toggles_membership_and_applies_counter_delta() {
    let user_id = uuid::Uuid::from_u128(1001);
    let target_id = uuid::Uuid::from_u128(1002);
    let mut pairs = std::collections::HashSet::new();

    let active =
        post::services::reactions::ReactionService::toggle_pair(&mut pairs, (user_id, target_id));
    assert!(active);
    assert!(pairs.contains(&(user_id, target_id)));

    let inactive =
        post::services::reactions::ReactionService::toggle_pair(&mut pairs, (user_id, target_id));
    assert!(!inactive);
    assert!(!pairs.contains(&(user_id, target_id)));

    let mut count = 0;
    assert_eq!(
        post::services::reactions::ReactionService::apply_counter_delta(&mut count, true),
        1
    );
    assert_eq!(
        post::services::reactions::ReactionService::apply_counter_delta(&mut count, false),
        0
    );
    assert_eq!(
        post::services::reactions::ReactionService::apply_counter_delta(&mut count, false),
        0
    );
}

#[test]
fn follow_service_rejects_self_follow_and_toggles_relationship() {
    let follower_id = uuid::Uuid::from_u128(1101);
    let followee_id = uuid::Uuid::from_u128(1102);
    let mut follows = std::collections::HashSet::new();

    let self_follow = post::services::follows::FollowService::toggle_follow(
        &mut follows,
        follower_id,
        follower_id,
    );
    assert_eq!(
        self_follow
            .expect_err("self follow should fail")
            .to_string(),
        "请求冲突: 不能关注自己"
    );

    let followed = post::services::follows::FollowService::toggle_follow(
        &mut follows,
        follower_id,
        followee_id,
    )
    .expect("follow");
    assert!(followed.following);
    assert_eq!(followed.follower_id, follower_id);
    assert_eq!(followed.followee_id, followee_id);
    assert!(follows.contains(&(follower_id, followee_id)));

    let unfollowed = post::services::follows::FollowService::toggle_follow(
        &mut follows,
        follower_id,
        followee_id,
    )
    .expect("unfollow");
    assert!(!unfollowed.following);
    assert!(!follows.contains(&(follower_id, followee_id)));
}

#[test]
fn content_moderation_routes_are_registered() {
    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/admin/posts"));
    assert!(routes.contains(&"/api/admin/posts/{post_id}/offline"));
    assert!(routes.contains(&"/api/admin/posts/{post_id}/restore"));
    assert!(routes.contains(&"/api/admin/posts/{post_id}/delete"));
    assert!(routes.contains(&"/api/admin/posts/{post_id}/pin"));
    assert!(routes.contains(&"/api/admin/posts/{post_id}/unpin"));
    assert!(routes.contains(&"/api/admin/comments"));
    assert!(routes.contains(&"/api/admin/comments/{comment_id}/delete"));
    assert!(routes.contains(&"/api/admin/comments/{comment_id}/recover"));
}

#[test]
fn comment_interaction_routes_are_registered() {
    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/comments/{comment_id}/delete"));
    assert!(routes.contains(&"/api/comments/{comment_id}/like"));
    assert!(routes.contains(&"/api/comments/{comment_id}/report"));
}

#[test]
fn user_admin_contract_supports_disable_enable_roles_and_audit_logs() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let member = store
        .login("managed-user", "password")
        .expect("login member")
        .user;

    assert!(store.admin_users(member.user_id).is_err());
    assert!(
        store
            .admin_users(admin.user_id)
            .expect("admin users")
            .iter()
            .any(|user| user.user_id == member.user_id && !user.disabled)
    );

    let context = post::domain::user_admin::AuditContext {
        ip: Some("127.0.0.1".to_string()),
        user_agent: Some("contract-test".to_string()),
    };
    let disabled = store
        .disable_user(admin.user_id, member.user_id, context.clone())
        .expect("disable user");
    assert!(disabled.disabled);
    assert!(store.login("managed-user", "password").is_err());

    let enabled = store
        .enable_user(admin.user_id, member.user_id, context.clone())
        .expect("enable user");
    assert!(!enabled.disabled);
    assert!(store.login("managed-user", "password").is_ok());

    let roles = store
        .update_user_roles(
            admin.user_id,
            member.user_id,
            post::domain::user_admin::UpdateUserRolesRequest {
                roles: vec!["moderator".to_string(), "operator".to_string()],
                context: context.clone(),
            },
        )
        .expect("update roles");
    assert_eq!(roles.roles, vec!["moderator", "operator"]);

    let logs = store.audit_logs(admin.user_id).expect("audit logs");
    assert!(logs.len() >= 3);
    assert!(logs.iter().any(|entry| {
        entry.actor_id == admin.user_id
            && entry.action == "user.disable"
            && entry.target_id == member.user_id
            && entry.ip.as_deref() == Some("127.0.0.1")
            && entry.user_agent.as_deref() == Some("contract-test")
            && entry.created_at <= time::OffsetDateTime::now_utc()
    }));
    assert!(logs.iter().any(|entry| {
        entry.action == "user.roles.update"
            && entry
                .after
                .as_deref()
                .is_some_and(|value| value.contains("moderator"))
    }));
}

#[test]
fn user_admin_service_builds_rows_roles_and_audit_entries() {
    let actor = post::domain::auth::SessionUser {
        user_id: uuid::Uuid::from_u128(1301),
        username: "admin".to_string(),
        nickname: "管理员".to_string(),
        avatar_url: None,
        is_admin: true,
    };
    let target = post::domain::auth::SessionUser {
        user_id: uuid::Uuid::from_u128(1302),
        username: "managed-user".to_string(),
        nickname: "被管理用户".to_string(),
        avatar_url: None,
        is_admin: false,
    };
    assert!(
        post::services::user_admin::UserAdminService::ensure_not_self_disable(
            actor.user_id,
            actor.user_id
        )
        .is_err()
    );

    let roles = post::services::user_admin::UserAdminService::normalize_roles(vec![
        " Moderator ".to_string(),
        "operator".to_string(),
        "moderator".to_string(),
        " ".to_string(),
    ])
    .expect("normalize roles");
    assert_eq!(roles, vec!["moderator", "operator"]);
    assert!(
        post::services::user_admin::UserAdminService::normalize_roles(vec![" ".to_string()])
            .is_err()
    );

    let row =
        post::services::user_admin::UserAdminService::admin_user_row(&target, roles, true, 3, 4);
    assert_eq!(row.username, "managed-user");
    assert!(row.disabled);
    assert_eq!(row.post_count, 3);
    assert_eq!(row.comment_count, 4);

    let snapshot = post::services::user_admin::UserAdminService::audit_snapshot(&row);
    assert!(snapshot.contains("disabled=true"));
    assert!(snapshot.contains("roles=moderator|operator"));

    let now = time::OffsetDateTime::now_utc();
    let audit = post::services::user_admin::UserAdminService::build_audit_log(
        uuid::Uuid::from_u128(1303),
        &actor,
        "user.disable",
        "user",
        target.user_id,
        target.nickname.clone(),
        None,
        Some(snapshot),
        post::domain::user_admin::AuditContext {
            ip: Some("127.0.0.1".to_string()),
            user_agent: Some("service-test".to_string()),
        },
        now,
    );
    assert_eq!(audit.actor_id, actor.user_id);
    assert_eq!(audit.target_id, target.user_id);
    assert_eq!(audit.ip.as_deref(), Some("127.0.0.1"));
    assert_eq!(audit.created_at, now);
}

#[test]
fn user_admin_routes_are_registered() {
    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/admin/users"));
    assert!(routes.contains(&"/api/admin/users/{user_id}/disable"));
    assert!(routes.contains(&"/api/admin/users/{user_id}/enable"));
    assert!(routes.contains(&"/api/admin/users/{user_id}/roles"));
    assert!(routes.contains(&"/api/admin/audit-logs"));
}

#[test]
fn rbac_contract_supports_role_and_permission_management() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let member = store.login("rbac-member", "password").expect("member").user;

    assert!(store.list_roles(member.user_id).is_err());
    assert!(store.list_permissions(member.user_id).is_err());

    let permissions = store
        .list_permissions(admin.user_id)
        .expect("admin permissions");
    for required in [
        "role:view",
        "role:create",
        "role:update",
        "role:delete",
        "permission:view",
    ] {
        assert!(
            permissions
                .iter()
                .any(|permission| permission.code == required),
            "missing permission {required}"
        );
    }

    let roles = store.list_roles(admin.user_id).expect("admin roles");
    for required in ["admin", "member", "moderator", "operator"] {
        assert!(roles.iter().any(|role| role.code == required));
    }

    let context = post::domain::user_admin::AuditContext {
        ip: Some("127.0.0.1".to_string()),
        user_agent: Some("rbac-contract".to_string()),
    };
    let reviewer = store
        .create_role(
            admin.user_id,
            post::domain::rbac::CreateRoleRequest {
                code: "reviewer".to_string(),
                name: "内容审核员".to_string(),
                permission_codes: vec![
                    "post:view".to_string(),
                    "comment:view".to_string(),
                    "comment:delete".to_string(),
                ],
                context: context.clone(),
            },
        )
        .expect("create role");
    assert_eq!(reviewer.code, "reviewer");
    assert_eq!(reviewer.permissions.len(), 3);

    assert!(
        store
            .create_role(
                admin.user_id,
                post::domain::rbac::CreateRoleRequest {
                    code: "reviewer".to_string(),
                    name: "重复角色".to_string(),
                    permission_codes: vec!["post:view".to_string()],
                    context: context.clone(),
                },
            )
            .is_err()
    );

    let updated = store
        .update_role(
            admin.user_id,
            "reviewer",
            post::domain::rbac::UpdateRoleRequest {
                name: Some("高级审核员".to_string()),
                permission_codes: Some(vec![
                    "post:view".to_string(),
                    "post:update".to_string(),
                    "report:view".to_string(),
                ]),
                context: context.clone(),
            },
        )
        .expect("update role");
    assert_eq!(updated.name, "高级审核员");
    assert!(
        updated
            .permissions
            .iter()
            .any(|permission| permission.code == "report:view")
    );

    store
        .update_user_roles(
            admin.user_id,
            member.user_id,
            post::domain::user_admin::UpdateUserRolesRequest {
                roles: vec!["reviewer".to_string()],
                context: context.clone(),
            },
        )
        .expect("assign reviewer");
    assert!(
        store
            .delete_role(admin.user_id, "reviewer", context.clone())
            .is_err()
    );

    let temporary = store
        .create_role(
            admin.user_id,
            post::domain::rbac::CreateRoleRequest {
                code: "temporary".to_string(),
                name: "临时角色".to_string(),
                permission_codes: vec!["user:view".to_string()],
                context: context.clone(),
            },
        )
        .expect("create temporary");
    assert_eq!(temporary.code, "temporary");
    let deleted = store
        .delete_role(admin.user_id, "temporary", context)
        .expect("delete temporary");
    assert_eq!(deleted.code, "temporary");
    assert!(
        store
            .list_roles(admin.user_id)
            .expect("roles after delete")
            .iter()
            .all(|role| role.code != "temporary")
    );
}

#[test]
fn rbac_service_normalizes_permissions_and_guards_builtin_roles() {
    let role = post::services::rbac::RbacService::build_role(
        "  Reviewer  ",
        "  内容审核员  ",
        &[
            "post:view".to_string(),
            " comment:delete ".to_string(),
            "post:view".to_string(),
        ],
    )
    .expect("build role");

    assert_eq!(role.code, "reviewer");
    assert_eq!(role.name, "内容审核员");
    assert_eq!(role.permissions.len(), 2);
    assert!(
        role.permissions
            .iter()
            .any(|permission| permission.code == "comment:delete")
    );

    let mut updated = role.clone();
    post::services::rbac::RbacService::apply_role_update(
        &mut updated,
        post::domain::rbac::UpdateRoleRequest {
            name: Some("  高级审核员  ".to_string()),
            permission_codes: Some(vec!["report:view".to_string(), "post:update".to_string()]),
            context: post::domain::user_admin::AuditContext::default(),
        },
    )
    .expect("update role");
    assert_eq!(updated.name, "高级审核员");
    assert_eq!(updated.permissions.len(), 2);
    assert!(
        updated
            .permissions
            .iter()
            .any(|permission| permission.code == "report:view")
    );

    assert!(
        post::services::rbac::RbacService::build_role(" ", "空角色", &["post:view".to_string()],)
            .is_err()
    );
    assert!(
        post::services::rbac::RbacService::build_role(
            "unknown-permission",
            "未知权限",
            &["missing:permission".to_string()],
        )
        .is_err()
    );
    assert!(post::services::rbac::RbacService::ensure_deletable_role("admin").is_err());
    post::services::rbac::RbacService::ensure_deletable_role("reviewer")
        .expect("custom role can be deleted");
}

#[test]
fn rbac_routes_are_registered() {
    let routes = post::app::api_route_inventory();
    assert!(routes.contains(&"/api/admin/roles"));
    assert!(routes.contains(&"/api/admin/roles/{role_code}/update"));
    assert!(routes.contains(&"/api/admin/roles/{role_code}/delete"));
    assert!(routes.contains(&"/api/admin/permissions"));
}

#[test]
fn dense_workbench_ui_exposes_prd_system_features() {
    let text = post::app::ui_feature_inventory();

    for required in [
        "关注动态",
        "分类过滤",
        "标签过滤",
        "分页",
        "Markdown 编辑",
        "图片上传",
        "实时预览",
        "代码高亮",
        "自动保存",
        "点赞",
        "收藏",
        "关注作者",
        "举报",
        "回复评论",
        "相关推荐",
        "WebSocket 推送",
        "消息中心",
        "全部已读",
        "全文搜索",
        "搜索高亮",
        "个人主页",
        "草稿",
        "RustFS",
        "NATS",
        "Elasticsearch",
        "RBAC",
        "用户管理",
        "角色管理",
        "权限管理",
        "帖子管理",
        "评论管理",
        "分类管理",
        "标签管理",
        "公告推送",
        "举报处理",
        "审计日志",
        "系统统计",
    ] {
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
    assert_eq!(
        store
            .comments_for_post(detail.summary.post_id)
            .unwrap()
            .len(),
        1
    );

    let liked = store
        .toggle_post_like(user.user_id, detail.summary.post_id)
        .unwrap();
    assert!(liked.active);
    assert_eq!(liked.count, 1);

    let favorited = store
        .toggle_post_favorite(user.user_id, detail.summary.post_id)
        .unwrap();
    assert!(favorited.active);
    assert_eq!(favorited.count, 1);

    let author = detail.summary.author_id;
    let followed = store.follow_user(user.user_id, author).unwrap_err();
    assert_eq!(followed.to_string(), "请求冲突: 不能关注自己");
}
