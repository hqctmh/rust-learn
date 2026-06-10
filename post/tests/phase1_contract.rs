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
        assert!(
            schema.contains(&format!("create table {table}")),
            "missing {table}"
        );
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

#[test]
fn prd_completion_supports_search_upload_notifications_and_announcements() {
    let store = post::state::ForumStore::seeded();
    let author = store.demo_user();
    let follower = store
        .register(post::domain::auth::RegisterRequest {
            username: "reader".to_string(),
            password: "secret123".to_string(),
            nickname: "读者".to_string(),
        })
        .expect("register follower")
        .user;

    store
        .follow_user(follower.user_id, author.user_id)
        .expect("follow author");

    let published = store
        .create_post(
            author.user_id,
            post::domain::posts::CreatePostRequest {
                title: "Elasticsearch 搜索索引设计".to_string(),
                markdown: "用 NATS 事件异步更新搜索索引。".to_string(),
                summary: "搜索、事件与索引同步。".to_string(),
                category_name: Some("Search".to_string()),
                tag_names: vec!["rust".to_string(), "elasticsearch".to_string()],
                publish: true,
            },
        )
        .expect("create followed post");

    let results = store.search_posts(post::domain::posts::SearchQuery {
        keyword: Some("索引".to_string()),
        category_name: Some("Search".to_string()),
        tag: Some("rust".to_string()),
        sort: post::domain::posts::SearchSort::Latest,
        page: 1,
        page_size: 10,
    });

    assert_eq!(results.total, 1);
    assert_eq!(results.items[0].post_id, published.summary.post_id);

    let follower_notifications = store.list_notifications(follower.user_id);
    assert!(follower_notifications.iter().any(|notification| {
        notification.notification_type
            == post::domain::notifications::NotificationType::FollowedUserPosted
            && notification.title.contains("Elasticsearch")
    }));

    let file = store
        .upload_file(
            author.user_id,
            post::domain::files::FileUploadRequest {
                original_filename: "cover.png".to_string(),
                size_bytes: 512 * 1024,
                mime_type: "image/png".to_string(),
                sha256: "abc123".to_string(),
                purpose: post::domain::files::FilePurpose::PostImage,
            },
        )
        .expect("upload image");

    assert_eq!(file.bucket, "post-images");
    assert!(file.public_url.ends_with("/cover.png"));

    let invalid_file = store
        .upload_file(
            author.user_id,
            post::domain::files::FileUploadRequest {
                original_filename: "payload.sh".to_string(),
                size_bytes: 1024,
                mime_type: "application/x-sh".to_string(),
                sha256: "danger".to_string(),
                purpose: post::domain::files::FilePurpose::PostImage,
            },
        )
        .unwrap_err();
    assert!(invalid_file.to_string().contains("不支持的文件类型"));

    store
        .publish_announcement(
            author.user_id,
            post::domain::notifications::AnnouncementRequest {
                title: "维护公告".to_string(),
                body: "今晚升级搜索索引。".to_string(),
                target: post::domain::notifications::AnnouncementTarget::AllUsers,
            },
        )
        .expect("publish announcement");

    let unread_count = store.mark_all_notifications_read(follower.user_id);
    assert!(unread_count >= 2);
    assert!(
        store
            .list_notifications(follower.user_id)
            .iter()
            .all(|notification| notification.read_at.is_some())
    );
}

#[test]
fn prd_completion_supports_reports_audit_logs_and_admin_stats() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let reporter = store
        .register(post::domain::auth::RegisterRequest {
            username: "reporter".to_string(),
            password: "secret123".to_string(),
            nickname: "举报者".to_string(),
        })
        .expect("register reporter")
        .user;
    let post = store.list_posts().remove(0);

    let report = store
        .create_report(post::domain::moderation::CreateReportRequest {
            reporter_id: reporter.user_id,
            target: post::domain::moderation::ReportTarget::Post(post.post_id),
            reason: "广告内容".to_string(),
            note: Some("疑似推广链接".to_string()),
        })
        .expect("create report");

    assert_eq!(report.status, post::domain::moderation::ReportStatus::Open);

    let resolved = store
        .resolve_report(
            admin.user_id,
            report.report_id,
            post::domain::moderation::ReportDecision::Resolved {
                action: post::domain::moderation::ModerationAction::TakePostOffline,
            },
        )
        .expect("resolve report");

    assert_eq!(
        resolved.status,
        post::domain::moderation::ReportStatus::Resolved
    );
    assert!(store.audit_logs().iter().any(|entry| {
        entry.actor_id == admin.user_id
            && entry.action == "report.resolve"
            && entry.target_id == Some(report.report_id)
    }));

    let stats = store.admin_stats();
    assert!(stats.user_total >= 2);
    assert!(stats.post_total >= 1);
    assert_eq!(stats.open_report_total, 0);
    assert!(stats.audit_log_total >= 1);
}

#[test]
fn prd_completion_api_routes_cover_remaining_prd_capabilities() {
    let routes = post::api::route_paths();

    for route in [
        "/api/search/posts",
        "/api/notifications",
        "/api/notifications/read-all",
        "/api/announcements",
        "/api/files",
        "/api/reports",
        "/api/admin/stats",
        "/api/admin/audit-logs",
        "/api/ws/notifications",
    ] {
        assert!(routes.contains(&route), "missing {route}");
    }
}

#[test]
fn websocket_notification_snapshot_serializes_unread_notifications() {
    let store = post::state::ForumStore::seeded();
    let author = store.demo_user();
    let follower = store
        .register(post::domain::auth::RegisterRequest {
            username: "ws-reader".to_string(),
            password: "secret123".to_string(),
            nickname: "WS 读者".to_string(),
        })
        .expect("register ws reader")
        .user;

    store
        .follow_user(follower.user_id, author.user_id)
        .expect("follow author");
    store
        .create_post(
            author.user_id,
            post::domain::posts::CreatePostRequest {
                title: "WebSocket 通知测试".to_string(),
                markdown: "发布后需要推送给关注者。".to_string(),
                summary: "验证 WebSocket 初始通知快照。".to_string(),
                category_name: Some("Notify".to_string()),
                tag_names: vec!["websocket".to_string()],
                publish: true,
            },
        )
        .expect("create post");

    let message = post::api::notification_ws_initial_message(&store, follower.user_id)
        .expect("serialize websocket message");

    assert!(message.contains("\"kind\":\"notification.snapshot\""));
    assert!(message.contains("WebSocket 通知测试"));
    assert!(message.contains("\"unread_count\":1"));
}

#[test]
fn notification_subscriber_receives_new_notifications_without_reconnect() {
    let store = post::state::ForumStore::seeded();
    let author = store.demo_user();
    let follower = store
        .register(post::domain::auth::RegisterRequest {
            username: "live-reader".to_string(),
            password: "secret123".to_string(),
            nickname: "实时读者".to_string(),
        })
        .expect("register live reader")
        .user;

    store
        .follow_user(follower.user_id, author.user_id)
        .expect("follow author");
    let receiver = store.subscribe_notifications(follower.user_id);

    store
        .create_post(
            author.user_id,
            post::domain::posts::CreatePostRequest {
                title: "实时通知测试".to_string(),
                markdown: "不重连也应该收到通知。".to_string(),
                summary: "验证通知广播。".to_string(),
                category_name: Some("Notify".to_string()),
                tag_names: vec!["websocket".to_string()],
                publish: true,
            },
        )
        .expect("create post");

    let notification = receiver
        .recv_timeout(std::time::Duration::from_secs(1))
        .expect("receive live notification");
    assert_eq!(notification.recipient_id, follower.user_id);
    assert_eq!(
        notification.notification_type,
        post::domain::notifications::NotificationType::FollowedUserPosted
    );
    assert!(notification.title.contains("实时通知测试"));
}

#[test]
fn forum_actions_record_prd_nats_events() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let session = store
        .register(post::domain::auth::RegisterRequest {
            username: "event-user".to_string(),
            password: "secret123".to_string(),
            nickname: "事件用户".to_string(),
        })
        .expect("register user");
    let user = session.user;

    store
        .follow_user(user.user_id, admin.user_id)
        .expect("follow");
    let post = store
        .create_post(
            admin.user_id,
            post::domain::posts::CreatePostRequest {
                title: "NATS 事件测试".to_string(),
                markdown: "发布后应记录 post.created 和 search.post.index。".to_string(),
                summary: "事件测试".to_string(),
                category_name: Some("Infra".to_string()),
                tag_names: vec!["nats".to_string()],
                publish: true,
            },
        )
        .expect("create post");
    store
        .add_comment(
            user.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: post.summary.post_id,
                parent_comment_id: None,
                content: "评论事件".to_string(),
            },
        )
        .expect("comment");
    store
        .toggle_post_like(user.user_id, post.summary.post_id)
        .expect("like post");
    store
        .publish_announcement(
            admin.user_id,
            post::domain::notifications::AnnouncementRequest {
                title: "事件公告".to_string(),
                body: "公告应产生 announcement.published。".to_string(),
                target: post::domain::notifications::AnnouncementTarget::AllUsers,
            },
        )
        .expect("announcement");

    let subjects = store
        .event_outbox()
        .iter()
        .map(|event| event.subject())
        .collect::<Vec<_>>();

    for subject in [
        "user.registered",
        "user.followed",
        "post.created",
        "search.post.index",
        "post.commented",
        "notification.created",
        "post.liked",
        "announcement.published",
    ] {
        assert!(subjects.contains(&subject), "missing {subject}");
    }
}

#[test]
fn nats_event_payload_contract_serializes_json_subject_and_event_type() {
    let event = post::domain::events::ForumEvent::SearchPostIndex {
        post_id: uuid::Uuid::from_u128(42),
        title: "索引标题".to_string(),
        body: "索引正文".to_string(),
        tags: vec!["rust".to_string()],
    };

    let payload = post::events::NatsEventPublisher::payload_json(&event).expect("serialize event");
    let value: serde_json::Value = serde_json::from_slice(&payload).expect("json payload");

    assert_eq!(event.subject(), "search.post.index");
    assert_eq!(value["event_type"], "search.post.index");
    assert_eq!(value["data"]["title"], "索引标题");
}

#[test]
fn elasticsearch_index_contract_maps_events_to_index_operations() {
    let post_id = uuid::Uuid::from_u128(101);
    let index_event = post::domain::events::ForumEvent::SearchPostIndex {
        post_id,
        title: "Rust 搜索".to_string(),
        body: "Elasticsearch 全文检索正文".to_string(),
        tags: vec!["rust".to_string(), "search".to_string()],
    };
    let delete_event = post::domain::events::ForumEvent::SearchPostDelete { post_id };

    let index_operation =
        post::search::SearchIndexOperation::from_event(&index_event).expect("index operation");
    let delete_operation =
        post::search::SearchIndexOperation::from_event(&delete_event).expect("delete operation");

    match index_operation {
        post::search::SearchIndexOperation::Index { document } => {
            assert_eq!(document.post_id, post_id);
            assert_eq!(document.title, "Rust 搜索");
            assert_eq!(
                document.tags,
                vec!["rust".to_string(), "search".to_string()]
            );
        }
        _ => panic!("expected index operation"),
    }
    assert_eq!(
        delete_operation,
        post::search::SearchIndexOperation::Delete { post_id }
    );
}

#[test]
fn elasticsearch_search_body_contract_uses_multi_match_filters_and_pagination() {
    let body =
        post::search::ElasticsearchPostIndexer::search_body(&post::domain::posts::SearchQuery {
            keyword: Some("Leptos Rust".to_string()),
            category_name: Some("Rust".to_string()),
            tag: Some("web".to_string()),
            sort: post::domain::posts::SearchSort::Hot,
            page: 2,
            page_size: 20,
        });

    assert_eq!(body["from"], 20);
    assert_eq!(body["size"], 20);
    assert_eq!(
        body["query"]["bool"]["must"][0]["multi_match"]["query"],
        "Leptos Rust"
    );
    assert_eq!(
        body["query"]["bool"]["must"][0]["multi_match"]["fields"],
        serde_json::json!(["title^3", "summary^2", "body", "tags", "category_name"])
    );
    assert_eq!(
        body["query"]["bool"]["filter"][0]["term"]["category_name.keyword"],
        "Rust"
    );
    assert_eq!(
        body["query"]["bool"]["filter"][1]["term"]["tags.keyword"],
        "web"
    );
    assert_eq!(body["sort"][0]["score"]["order"], "desc");
}

#[test]
fn rustfs_object_upload_contract_builds_bucket_key_and_body() {
    let user_id = uuid::Uuid::from_u128(77);
    let request = post::domain::files::FileUploadRequest {
        original_filename: "../cover.png".to_string(),
        size_bytes: 4,
        mime_type: "image/png".to_string(),
        sha256: "abc123".to_string(),
        purpose: post::domain::files::FilePurpose::PostCover,
    };

    let upload = post::storage::ObjectUpload::from_file_request(
        user_id,
        &request,
        bytes::Bytes::from_static(b"rust"),
    )
    .expect("object upload");

    assert_eq!(upload.bucket, "post-images");
    assert_eq!(upload.object_key, format!("{user_id}/abc123/cover.png"));
    assert_eq!(upload.content_type, "image/png");
    assert_eq!(upload.body, bytes::Bytes::from_static(b"rust"));
    assert_eq!(
        upload.public_url(),
        format!("/files/post-images/{user_id}/abc123/cover.png")
    );
}

#[test]
fn file_metadata_upload_contract_deduplicates_same_hash_for_user() {
    let store = post::state::ForumStore::seeded();
    let user = store.demo_user();
    let request = post::domain::files::FileUploadRequest {
        original_filename: "cover.png".to_string(),
        size_bytes: 512,
        mime_type: "image/png".to_string(),
        sha256: "same-hash".to_string(),
        purpose: post::domain::files::FilePurpose::PostImage,
    };

    let first = store
        .upload_file(user.user_id, request.clone())
        .expect("first upload");
    let second = store
        .upload_file(user.user_id, request)
        .expect("duplicate upload");

    assert_eq!(first.file_id, second.file_id);
    assert_eq!(first.object_key, second.object_key);
    assert_eq!(store.admin_stats().file_total, 1);
}

#[test]
fn auth_session_contract_registers_resolves_and_logs_out_user() {
    let store = post::state::ForumStore::seeded();

    let session = store
        .register(post::domain::auth::RegisterRequest {
            username: "new-author".to_string(),
            password: "secret123".to_string(),
            nickname: "新作者".to_string(),
        })
        .expect("register user");

    assert_eq!(session.user.username, "new-author");
    assert_eq!(session.user.nickname, "新作者");

    let current_user = store
        .current_user(session.session_id)
        .expect("session should resolve");
    assert_eq!(current_user.user_id, session.user.user_id);

    let duplicate = store
        .register(post::domain::auth::RegisterRequest {
            username: "new-author".to_string(),
            password: "another-secret".to_string(),
            nickname: "重复用户".to_string(),
        })
        .unwrap_err();
    assert!(duplicate.to_string().contains("用户名已存在"));

    store.logout(session.session_id).expect("logout");
    let expired = store.current_user(session.session_id).unwrap_err();
    assert_eq!(expired.to_string(), "请先登录");
}

#[test]
fn auth_api_routes_cover_register_current_user_and_logout() {
    let routes = post::api::route_paths();

    for route in ["/api/register", "/api/me", "/api/logout"] {
        assert!(routes.contains(&route), "missing {route}");
    }
}

#[test]
fn auth_password_contract_requires_registered_user_and_hashes_password() {
    let store = post::state::ForumStore::seeded();

    let unknown = store.login("ghost", "secret123").unwrap_err();
    assert_eq!(unknown.to_string(), "请先登录");

    store
        .register(post::domain::auth::RegisterRequest {
            username: "hash-user".to_string(),
            password: "secret123".to_string(),
            nickname: "哈希用户".to_string(),
        })
        .expect("register user");

    let password_hash = store
        .password_hash_for_user("hash-user")
        .expect("password hash");
    assert_ne!(password_hash, "secret123");
    assert!(password_hash.starts_with("$argon2"));

    let wrong_password = store.login("hash-user", "wrong-password").unwrap_err();
    assert_eq!(wrong_password.to_string(), "请先登录");

    let login = store.login("hash-user", "secret123").expect("login");
    assert_eq!(login.user.username, "hash-user");
}

#[test]
fn rbac_contract_blocks_regular_user_and_allows_admin_permissions() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let regular = store
        .register(post::domain::auth::RegisterRequest {
            username: "regular-user".to_string(),
            password: "secret123".to_string(),
            nickname: "普通用户".to_string(),
        })
        .expect("register regular user")
        .user;

    let admin_permissions = store
        .permissions_for_user(admin.user_id)
        .expect("admin permissions");
    for code in [
        "announcement:publish",
        "report:resolve",
        "stats:view",
        "audit:view",
    ] {
        assert!(
            admin_permissions
                .iter()
                .any(|permission| permission.code == code),
            "missing admin permission {code}"
        );
        store
            .require_permission(admin.user_id, code)
            .expect("admin permission");
    }

    let denied = store
        .require_permission(regular.user_id, "announcement:publish")
        .unwrap_err();
    assert_eq!(denied.to_string(), "没有权限执行该操作");

    assert!(
        store
            .permissions_for_user(regular.user_id)
            .expect("regular permissions")
            .is_empty()
    );
}

#[test]
fn admin_api_permission_contract_requires_session_and_permission() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let regular = store
        .register(post::domain::auth::RegisterRequest {
            username: "api-regular".to_string(),
            password: "secret123".to_string(),
            nickname: "API 普通用户".to_string(),
        })
        .expect("register regular")
        .user;

    let regular_session = store.login("api-regular", "secret123").expect("login");
    let admin_session = store.login("mah", "demo-password").expect("admin login");

    let missing_session =
        post::api::authorize_session_for_permission(&store, None, "announcement:publish")
            .unwrap_err();
    assert_eq!(missing_session.to_string(), "请先登录");

    let forbidden = post::api::authorize_session_for_permission(
        &store,
        Some(regular_session.session_id),
        "announcement:publish",
    )
    .unwrap_err();
    assert_eq!(forbidden.to_string(), "没有权限执行该操作");

    let allowed = post::api::authorize_session_for_permission(
        &store,
        Some(admin_session.session_id),
        "announcement:publish",
    )
    .expect("admin allowed");
    assert_eq!(allowed.user_id, admin.user_id);
    assert_ne!(allowed.user_id, regular.user_id);
}

#[test]
fn user_action_api_contract_requires_valid_session() {
    let store = post::state::ForumStore::seeded();
    let session = store
        .register(post::domain::auth::RegisterRequest {
            username: "session-author".to_string(),
            password: "secret123".to_string(),
            nickname: "会话作者".to_string(),
        })
        .expect("register user");

    let missing = post::api::authorize_session_user(&store, None).unwrap_err();
    assert_eq!(missing.to_string(), "请先登录");

    let invalid =
        post::api::authorize_session_user(&store, Some(uuid::Uuid::from_u128(9_999))).unwrap_err();
    assert_eq!(invalid.to_string(), "请先登录");

    let authorized =
        post::api::authorize_session_user(&store, Some(session.session_id)).expect("valid session");
    assert_eq!(authorized.user_id, session.user.user_id);

    store.logout(session.session_id).expect("logout");
    let logged_out =
        post::api::authorize_session_user(&store, Some(session.session_id)).unwrap_err();
    assert_eq!(logged_out.to_string(), "请先登录");
}

#[test]
fn user_profile_and_owner_content_crud_contract() {
    let store = post::state::ForumStore::seeded();
    let author = store
        .register(post::domain::auth::RegisterRequest {
            username: "owner-author".to_string(),
            password: "secret123".to_string(),
            nickname: "原昵称".to_string(),
        })
        .expect("register author")
        .user;
    let other = store
        .register(post::domain::auth::RegisterRequest {
            username: "other-author".to_string(),
            password: "secret123".to_string(),
            nickname: "其他作者".to_string(),
        })
        .expect("register other")
        .user;

    let profile = store
        .update_profile(
            author.user_id,
            post::domain::auth::UpdateProfileRequest {
                nickname: "新昵称".to_string(),
                avatar_url: Some("/files/avatars/me.png".to_string()),
                bio: "Rust 与 Leptos 作者".to_string(),
            },
        )
        .expect("update profile");
    assert_eq!(profile.nickname, "新昵称");
    assert_eq!(profile.bio, "Rust 与 Leptos 作者");

    let post = store
        .create_post(
            author.user_id,
            post::domain::posts::CreatePostRequest {
                title: "可编辑帖子".to_string(),
                markdown: "初始正文".to_string(),
                summary: "初始摘要".to_string(),
                category_name: Some("Rust".to_string()),
                tag_names: vec!["leptos".to_string()],
                publish: true,
            },
        )
        .expect("create post");

    let forbidden = store
        .update_post(
            other.user_id,
            post.summary.post_id,
            post::domain::posts::UpdatePostRequest {
                title: "越权修改".to_string(),
                markdown: "不应该成功".to_string(),
                summary: "越权".to_string(),
                category_name: Some("Rust".to_string()),
                tag_names: vec![],
                publish: true,
            },
        )
        .unwrap_err();
    assert_eq!(forbidden.to_string(), "没有权限执行该操作");

    let updated = store
        .update_post(
            author.user_id,
            post.summary.post_id,
            post::domain::posts::UpdatePostRequest {
                title: "已编辑帖子".to_string(),
                markdown: "更新后的正文".to_string(),
                summary: "更新摘要".to_string(),
                category_name: Some("Leptos".to_string()),
                tag_names: vec!["rust".to_string(), "web".to_string()],
                publish: true,
            },
        )
        .expect("owner update");
    assert_eq!(updated.summary.title, "已编辑帖子");
    assert!(updated.sanitized_html.contains("更新后的正文"));

    let comment = store
        .add_comment(
            author.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: post.summary.post_id,
                parent_comment_id: None,
                content: "自己的评论".to_string(),
            },
        )
        .expect("create comment");

    let other_delete = store
        .delete_comment(other.user_id, comment.comment_id)
        .unwrap_err();
    assert_eq!(other_delete.to_string(), "没有权限执行该操作");

    store
        .delete_comment(author.user_id, comment.comment_id)
        .expect("owner delete comment");
    let comments = store.comments_for_post(post.summary.post_id).unwrap();
    assert!(comments[0].deleted);
    assert_eq!(comments[0].content, "该评论已被删除");

    store
        .delete_post(author.user_id, post.summary.post_id)
        .expect("owner delete post");
    let deleted = store
        .post_detail(post.summary.post_id)
        .expect("deleted detail");
    assert_eq!(deleted.status, post::domain::posts::PostStatus::Deleted);
    assert!(
        !store
            .list_posts()
            .iter()
            .any(|summary| summary.post_id == post.summary.post_id)
    );
}

#[test]
fn admin_management_contract_covers_users_content_and_taxonomy() {
    let store = post::state::ForumStore::seeded();
    let admin = store.demo_user();
    let regular = store
        .register(post::domain::auth::RegisterRequest {
            username: "managed-user".to_string(),
            password: "secret123".to_string(),
            nickname: "被管理用户".to_string(),
        })
        .expect("register regular")
        .user;

    let category = store
        .create_category(
            admin.user_id,
            post::domain::taxonomy::TaxonomyInput {
                name: "后端".to_string(),
                slug: "backend".to_string(),
                sort_order: 10,
            },
        )
        .expect("create category");
    let tag = store
        .create_tag(
            admin.user_id,
            post::domain::taxonomy::TagInput {
                name: "Leptos".to_string(),
                slug: "leptos".to_string(),
            },
        )
        .expect("create tag");
    assert_eq!(category.slug, "backend");
    assert_eq!(tag.slug, "leptos");

    let post = store
        .create_post(
            regular.user_id,
            post::domain::posts::CreatePostRequest {
                title: "待审核帖子".to_string(),
                markdown: "正文".to_string(),
                summary: "摘要".to_string(),
                category_name: Some(category.name.clone()),
                tag_names: vec![tag.name.clone()],
                publish: true,
            },
        )
        .expect("create post");
    let comment = store
        .add_comment(
            regular.user_id,
            post::domain::comments::CreateCommentRequest {
                post_id: post.summary.post_id,
                parent_comment_id: None,
                content: "待审核评论".to_string(),
            },
        )
        .expect("create comment");

    let disabled = store
        .set_user_disabled(admin.user_id, regular.user_id, true)
        .expect("disable user");
    assert_eq!(disabled.status, post::domain::auth::UserStatus::Disabled);

    let offline = store
        .set_post_status(
            admin.user_id,
            post.summary.post_id,
            post::domain::posts::PostStatus::Offline,
        )
        .expect("take post offline");
    assert_eq!(offline.status, post::domain::posts::PostStatus::Offline);

    store
        .admin_delete_comment(admin.user_id, comment.comment_id)
        .expect("admin delete comment");
    assert!(store.comments_for_post(post.summary.post_id).unwrap()[0].deleted);

    let users = store.admin_users();
    assert!(users.iter().any(|user| user.user_id == regular.user_id));
    assert!(store.categories().iter().any(|item| item.slug == "backend"));
    assert!(store.tags().iter().any(|item| item.slug == "leptos"));
    assert!(
        store
            .audit_logs()
            .iter()
            .any(|entry| { entry.actor_id == admin.user_id && entry.action == "user.disable" })
    );
}

#[test]
fn admin_and_crud_api_routes_cover_prd_management_surface() {
    let routes = post::api::route_paths();

    for route in [
        "/api/users/me/profile",
        "/api/posts/{post_id}",
        "/api/comments/{comment_id}",
        "/api/categories",
        "/api/tags",
        "/api/admin/users",
        "/api/admin/users/{user_id}/disabled",
        "/api/admin/posts/{post_id}/status",
        "/api/admin/comments/{comment_id}",
        "/api/admin/categories",
        "/api/admin/tags",
    ] {
        assert!(routes.contains(&route), "missing {route}");
    }
}
