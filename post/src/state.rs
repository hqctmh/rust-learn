use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    domain::{
        admin::{AdminDashboard, admin_dashboard_demo},
        announcements::{
            AnnouncementAudience, AnnouncementItem, AnnouncementReadState, AnnouncementStatus,
            CreateAnnouncementRequest, UpdateAnnouncementRequest,
        },
        auth::{RegisterRequest, Session, SessionUser},
        comments::{CommentNode, CommentPage, CommentPageQuery, CreateCommentRequest},
        files::{FileAsset, FileBinaryUploadRequest, FileUploadRequest, build_file_asset},
        home::{
            HomeAnnouncement, HomeCategory, HomePageData, HomeQuery, HomeTag, dense_workbench_home,
        },
        integrations::{
            IntegrationAction, announcement_published_actions, post_comment_changed_actions,
            post_published_actions,
        },
        moderation::{
            ModerationCommentAction, ModerationCommentRow, ModerationPostAction, ModerationPostRow,
        },
        notifications::{
            Notification, NotificationCenter, NotificationConnectionStats, NotificationPush,
            NotificationType, unread_count,
        },
        posts::{
            AutosaveDraftRequest, CreatePostRequest, PostDetail, PostStatus, PostSummary,
            UpdatePostRequest,
        },
        rbac::{CreateRoleRequest, Permission, Role, UpdateRoleRequest, admin_permissions},
        reactions::{FollowState, ToggleResult},
        reports::{CreateReportRequest, HandleReportRequest, ReportItem, ReportTargetType},
        search::{SearchQuery, SearchResultPage, search_dense_workbench},
        taxonomy::{
            CategoryItem, CreateCategoryRequest, CreateTagRequest, MergeTagRequest, TagItem,
            UpdateCategoryRequest, UpdateTagRequest,
        },
        user_admin::{AdminUserRow, AuditContext, AuditLogEntry, UpdateUserRolesRequest},
        users::{
            ChangePasswordRequest, UpdateAvatarRequest, UpdateProfileRequest, UserCommentItem,
            UserProfile, UserSpace, UserStats,
        },
    },
    error::ForumError,
    services::{
        announcements::AnnouncementService, auth::AuthService, comments::CommentService,
        follows::FollowService, moderation::ModerationService,
        notifications::NotificationPushService, posts::PostAuthoringService, rbac::RbacService,
        reactions::ReactionService, reports::ReportService, taxonomy::TaxonomyService,
        user_admin::UserAdminService, users::UserSettingsService,
    },
};

#[cfg(feature = "ssr")]
use sqlx::PgPool;

#[cfg(feature = "ssr")]
use crate::domain::{
    admin::{
        AdminAnnouncementRow, AdminCategoryRow, AdminCommentRow, AdminPostRow, AdminReportRow,
        AdminStat, AdminTagRow, AdminUserRow as DashboardUserRow, AuditEntry,
    },
    reports::ReportStatus,
};

#[cfg(feature = "ssr")]
use crate::object_store::{RustfsObjectStore, RustfsObjectStoreConfig};

#[cfg(feature = "ssr")]
use crate::repositories::{
    admin_audit::PostgresAdminAuditRepository,
    admin_stats::PostgresAdminStatsRepository,
    announcements::PostgresAnnouncementRepository,
    auth::PostgresAuthRepository,
    comments::PostgresCommentRepository,
    files::PostgresFileRepository,
    follows::PostgresFollowRepository,
    home::PostgresHomeRepository,
    integrations::PostgresIntegrationRepository,
    moderation::PostgresModerationRepository,
    notifications::PostgresNotificationRepository,
    posts::PostgresPostRepository,
    rbac::PostgresRbacRepository,
    reactions::PostgresReactionRepository,
    reports::PostgresReportRepository,
    search::{ElasticsearchSearchRepository, PostgresSearchRepository},
    taxonomy::PostgresTaxonomyRepository,
    user_admin::PostgresUserAdminRepository,
    users::PostgresUserSettingsRepository,
};

#[cfg(feature = "ssr")]
use crate::repositories::home::{HomeSidebarSnapshot, RedisHomeCacheRepository};

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AppState {
    pub db: Option<PgPool>,
    pub forum: ForumStore,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub fn uses_postgres_auth(&self) -> bool {
        self.db.is_some()
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<Session, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.login(username, password);
        };

        let login = AuthService::normalize_login(username, password)?;
        let user = PostgresAuthRepository::find_user_by_username(pool, &login.username)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }
        AuthService::validate_password_match(&user.password_hash, &login.password)?;

        let session_id = Uuid::new_v4();
        let session =
            AuthService::build_session(session_id, user.session_user(), OffsetDateTime::now_utc());
        PostgresAuthRepository::insert_session(
            pool,
            session.session_id,
            session.user.user_id,
            &session.session_id.to_string(),
            session.expires_at,
        )
        .await
        .map_err(|_| ForumError::Internal)?;

        Ok(session)
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<Session, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.register(request);
        };

        let registration = AuthService::normalize_registration(request)?;
        let user = PostgresAuthRepository::insert_user(
            pool,
            Uuid::new_v4(),
            &registration.username,
            &AuthService::hash_password(&registration.password),
            &registration.nickname,
            None,
            false,
        )
        .await
        .map_err(map_user_insert_error)?;

        let session_id = Uuid::new_v4();
        let session =
            AuthService::build_session(session_id, user.session_user(), OffsetDateTime::now_utc());
        PostgresAuthRepository::insert_session(
            pool,
            session.session_id,
            session.user.user_id,
            &session.session_id.to_string(),
            session.expires_at,
        )
        .await
        .map_err(|_| ForumError::Internal)?;

        Ok(session)
    }

    pub async fn current_session(&self, session_id: Uuid) -> Result<Session, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.current_session(session_id);
        };

        let row = PostgresAuthRepository::find_session(pool, session_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if row.is_disabled() {
            return Err(ForumError::Forbidden);
        }
        let session = row.session();
        AuthService::validate_session_active(session.expires_at, OffsetDateTime::now_utc())?;

        Ok(session)
    }

    pub async fn logout(&self, session_id: Uuid) -> Result<Session, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.logout(session_id);
        };

        let session = self.current_session(session_id).await?;
        PostgresAuthRepository::delete_session(pool, session_id)
            .await
            .map_err(|_| ForumError::Internal)?;

        Ok(session)
    }

    pub async fn list_posts(&self) -> Result<Vec<PostSummary>, ForumError> {
        let Some(pool) = &self.db else {
            return Ok(self.forum.list_posts());
        };

        PostgresPostRepository::list_published_summaries(pool, 50, 0)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn post_detail(&self, post_id: Uuid) -> Result<PostDetail, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.post_detail(post_id);
        };

        let mut detail = PostgresPostRepository::find_detail(pool, post_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if detail.status == PostStatus::Published {
            if let Some(view_count) = PostgresPostRepository::increment_view_count(pool, post_id)
                .await
                .map_err(|_| ForumError::Internal)?
            {
                detail.summary.view_count = view_count;
            }
        }
        Ok(detail)
    }

    pub async fn post_detail_for_user(
        &self,
        post_id: Uuid,
        current_user_id: Option<Uuid>,
    ) -> Result<PostDetail, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.post_detail_for_user(post_id, current_user_id);
        };

        let mut detail = self.post_detail(post_id).await?;
        if let Some(user_id) = current_user_id {
            let user = PostgresAuthRepository::find_user_by_id(pool, user_id)
                .await
                .map_err(|_| ForumError::Internal)?
                .ok_or(ForumError::Unauthorized)?;
            if user.is_disabled() {
                return Err(ForumError::Forbidden);
            }
            if detail.status == PostStatus::Published {
                PostgresPostRepository::mark_post_read(pool, user_id, post_id)
                    .await
                    .map_err(|_| ForumError::Internal)?;
                detail.summary.read_by_me = true;
            }
        }
        Ok(detail)
    }

    pub async fn related_posts_for_post(
        &self,
        post_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PostSummary>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.related_posts_for_post(post_id, limit as usize);
        };

        PostgresPostRepository::list_related_summaries(pool, post_id, limit, None)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn comments_for_post(&self, post_id: Uuid) -> Result<Vec<CommentNode>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.comments_for_post(post_id);
        };

        PostgresCommentRepository::list_for_post(pool, post_id)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn comments_page_for_post(
        &self,
        post_id: Uuid,
        query: CommentPageQuery,
    ) -> Result<CommentPage, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.comments_page_for_post(post_id, query);
        };

        PostgresCommentRepository::page_for_post(pool, post_id, query)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn create_post(
        &self,
        author_id: Uuid,
        request: CreatePostRequest,
    ) -> Result<PostDetail, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.create_post(author_id, request);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, author_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }

        let detail = PostAuthoringService::build_post(
            Uuid::new_v4(),
            &user.session_user(),
            request,
            OffsetDateTime::now_utc(),
        )?;
        PostgresPostRepository::insert_post(pool, &detail)
            .await
            .map_err(|_| ForumError::Internal)?;

        if detail.status == PostStatus::Published {
            PostgresIntegrationRepository::insert_actions(pool, &post_published_actions(&detail))
                .await
                .map_err(|_| ForumError::Internal)?;
            let followers = PostgresNotificationRepository::followers_for_user(pool, author_id)
                .await
                .map_err(|_| ForumError::Internal)?;
            for follower_id in followers {
                postgres_insert_notification(
                    pool,
                    follower_id,
                    Some(author_id),
                    NotificationType::FollowedUserPosted,
                    format!("{} 发布了新帖子", user.nickname),
                    detail.summary.title.clone(),
                )
                .await?;
            }
        }

        Ok(detail)
    }

    pub async fn update_post(
        &self,
        author_id: Uuid,
        post_id: Uuid,
        request: UpdatePostRequest,
    ) -> Result<PostDetail, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.update_post(author_id, post_id, request);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, author_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }

        let mut detail = PostgresPostRepository::find_detail(pool, post_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if detail.summary.author_id != author_id {
            return Err(ForumError::Forbidden);
        }
        PostAuthoringService::apply_update(&mut detail, request, OffsetDateTime::now_utc())?;
        PostgresPostRepository::update_post(pool, &detail)
            .await
            .map_err(|_| ForumError::Internal)?;

        Ok(detail)
    }

    pub async fn autosave_draft(
        &self,
        author_id: Uuid,
        request: AutosaveDraftRequest,
    ) -> Result<PostDetail, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.autosave_draft(author_id, request);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, author_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }

        if let Some(post_id) = request.post_id {
            let mut detail = PostgresPostRepository::find_detail(pool, post_id)
                .await
                .map_err(|_| ForumError::Internal)?
                .ok_or(ForumError::NotFound)?;
            if detail.summary.author_id != author_id {
                return Err(ForumError::Forbidden);
            }
            if detail.status != PostStatus::Draft {
                return Err(ForumError::Conflict("只能自动保存草稿".to_string()));
            }

            PostAuthoringService::apply_autosave(&mut detail, request)?;
            PostgresPostRepository::update_post(pool, &detail)
                .await
                .map_err(|_| ForumError::Internal)?;
            return Ok(detail);
        }

        let detail =
            PostAuthoringService::build_draft(Uuid::new_v4(), &user.session_user(), request)?;
        PostgresPostRepository::insert_post(pool, &detail)
            .await
            .map_err(|_| ForumError::Internal)?;

        Ok(detail)
    }

    pub async fn delete_own_post(
        &self,
        author_id: Uuid,
        post_id: Uuid,
    ) -> Result<PostDetail, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.delete_own_post(author_id, post_id);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, author_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }

        let mut detail = PostgresPostRepository::find_detail(pool, post_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if detail.summary.author_id != author_id {
            return Err(ForumError::Forbidden);
        }
        let affected = PostgresPostRepository::mark_deleted(pool, post_id, author_id)
            .await
            .map_err(|_| ForumError::Internal)?;
        if affected == 0 {
            return Err(ForumError::NotFound);
        }
        detail.status = PostStatus::Deleted;

        Ok(detail)
    }

    pub async fn add_comment(
        &self,
        author_id: Uuid,
        request: CreateCommentRequest,
    ) -> Result<CommentNode, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.add_comment(author_id, request);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, author_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }

        let post = PostgresPostRepository::find_detail(pool, request.post_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        let comment = CommentService::build_comment(
            Uuid::new_v4(),
            request.post_id,
            request.parent_comment_id,
            author_id,
            &user.nickname,
            post.summary.author_id,
            &request.content,
            OffsetDateTime::now_utc(),
        )?;

        PostgresCommentRepository::insert_comment(pool, &comment)
            .await
            .map_err(|_| ForumError::Internal)?;
        let post_snapshot = PostgresPostRepository::find_detail(pool, request.post_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        let actions = post_comment_changed_actions(&post_snapshot, comment.comment_id);
        PostgresIntegrationRepository::insert_actions(pool, &actions)
            .await
            .map_err(|_| ForumError::Internal)?;

        if author_id != post.summary.author_id {
            postgres_insert_notification(
                pool,
                post.summary.author_id,
                Some(author_id),
                if request.parent_comment_id.is_some() {
                    NotificationType::CommentReplied
                } else {
                    NotificationType::PostCommented
                },
                format!("{} 评论了你的帖子", user.nickname),
                CommentService::notification_body(&comment.content),
            )
            .await?;
        }

        Ok(comment)
    }

    pub async fn delete_own_comment(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<CommentNode, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.delete_own_comment(user_id, comment_id);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, user_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }

        let comment = PostgresCommentRepository::find_by_id(pool, comment_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if comment.author_id != user_id {
            return Err(ForumError::Forbidden);
        }

        PostgresCommentRepository::mark_deleted(pool, &comment)
            .await
            .map_err(|_| ForumError::Internal)?;

        let mut deleted = comment;
        deleted.deleted = true;
        deleted.content = "该评论已被删除".to_string();
        Ok(deleted)
    }

    pub async fn toggle_comment_like(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<ToggleResult, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.toggle_comment_like(user_id, comment_id);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, user_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }

        let comment = PostgresCommentRepository::find_by_id(pool, comment_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if comment.deleted {
            return Err(ForumError::Conflict("已删除评论不能点赞".to_string()));
        }

        PostgresCommentRepository::toggle_like(pool, user_id, comment_id)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn toggle_post_like(
        &self,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<ToggleResult, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.toggle_post_like(user_id, post_id);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, user_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }
        PostgresPostRepository::find_detail(pool, post_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;

        PostgresReactionRepository::toggle_post_like(pool, user_id, post_id)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn toggle_post_favorite(
        &self,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<ToggleResult, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.toggle_post_favorite(user_id, post_id);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, user_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }
        PostgresPostRepository::find_detail(pool, post_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;

        PostgresReactionRepository::toggle_post_favorite(pool, user_id, post_id)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn follow_user(
        &self,
        follower_id: Uuid,
        followee_id: Uuid,
    ) -> Result<FollowState, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.follow_user(follower_id, followee_id);
        };

        if follower_id == followee_id {
            return Err(ForumError::Conflict("不能关注自己".to_string()));
        }

        let follower = PostgresAuthRepository::find_user_by_id(pool, follower_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        let followee = PostgresAuthRepository::find_user_by_id(pool, followee_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if follower.is_disabled() || followee.is_disabled() {
            return Err(ForumError::Forbidden);
        }

        PostgresFollowRepository::toggle_follow(pool, follower_id, followee_id)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn create_report(
        &self,
        reporter_id: Uuid,
        request: CreateReportRequest,
    ) -> Result<ReportItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.create_report(reporter_id, request);
        };

        let reporter = PostgresAuthRepository::find_user_by_id(pool, reporter_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if reporter.is_disabled() {
            return Err(ForumError::Forbidden);
        }

        let target_title = postgres_report_target_title(pool, &request).await?;
        let report = ReportService::build_report(
            Uuid::new_v4(),
            reporter_id,
            &reporter.nickname,
            target_title,
            request,
            OffsetDateTime::now_utc(),
        )?;
        PostgresReportRepository::insert_report(pool, &report)
            .await
            .map_err(|_| ForumError::Internal)?;

        Ok(report)
    }

    pub async fn report_comment(
        &self,
        reporter_id: Uuid,
        comment_id: Uuid,
        mut request: CreateReportRequest,
    ) -> Result<ReportItem, ForumError> {
        request.target_type = ReportTargetType::Comment;
        request.target_id = comment_id;
        self.create_report(reporter_id, request).await
    }

    pub async fn list_reports(&self, admin_id: Uuid) -> Result<Vec<ReportItem>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.list_reports(admin_id);
        };

        postgres_admin_user(pool, admin_id).await?;
        PostgresReportRepository::list_reports(pool)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn handle_report(
        &self,
        admin_id: Uuid,
        report_id: Uuid,
        request: HandleReportRequest,
    ) -> Result<ReportItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.handle_report(admin_id, report_id, request);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let mut report = PostgresReportRepository::find_report(pool, report_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        ReportService::apply_handle(
            &mut report,
            admin.user_id,
            &admin.nickname,
            request,
            OffsetDateTime::now_utc(),
        )?;
        let rows_affected = PostgresReportRepository::update_report_handle(pool, &report)
            .await
            .map_err(|_| ForumError::Internal)?;
        if rows_affected == 0 {
            return Err(ForumError::NotFound);
        }
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            "report.handle",
            "report",
            report.report_id,
            &report_audit_snapshot(&report),
        )
        .await
        .map_err(|_| ForumError::Internal)?;

        Ok(report)
    }

    pub async fn create_announcement(
        &self,
        admin_id: Uuid,
        request: CreateAnnouncementRequest,
    ) -> Result<AnnouncementItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.create_announcement(admin_id, request);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let now = OffsetDateTime::now_utc();
        let announcement = AnnouncementService::build_draft(
            Uuid::new_v4(),
            admin.user_id,
            &admin.nickname,
            request,
            now,
        )?;
        PostgresAnnouncementRepository::insert_announcement(pool, &announcement)
            .await
            .map_err(|_| ForumError::Internal)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            "announcement.create",
            "announcement",
            announcement.announcement_id,
            &announcement_audit_snapshot(&announcement),
        )
        .await
        .map_err(|_| ForumError::Internal)?;

        Ok(announcement)
    }

    pub async fn update_announcement(
        &self,
        admin_id: Uuid,
        announcement_id: Uuid,
        request: UpdateAnnouncementRequest,
    ) -> Result<AnnouncementItem, ForumError> {
        let Some(pool) = &self.db else {
            return self
                .forum
                .update_announcement(admin_id, announcement_id, request);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let mut announcement =
            PostgresAnnouncementRepository::find_announcement(pool, announcement_id)
                .await
                .map_err(|_| ForumError::Internal)?
                .ok_or(ForumError::NotFound)?;
        AnnouncementService::apply_update(&mut announcement, request, OffsetDateTime::now_utc())?;
        let rows_affected =
            PostgresAnnouncementRepository::update_announcement(pool, &announcement)
                .await
                .map_err(|_| ForumError::Internal)?;
        if rows_affected == 0 {
            return Err(ForumError::NotFound);
        }
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            "announcement.update",
            "announcement",
            announcement.announcement_id,
            &announcement_audit_snapshot(&announcement),
        )
        .await
        .map_err(|_| ForumError::Internal)?;

        Ok(announcement)
    }

    pub async fn list_admin_announcements(
        &self,
        admin_id: Uuid,
    ) -> Result<Vec<AnnouncementItem>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.list_admin_announcements(admin_id);
        };

        postgres_admin_user(pool, admin_id).await?;
        PostgresAnnouncementRepository::list_admin_announcements(pool)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn publish_announcement(
        &self,
        admin_id: Uuid,
        announcement_id: Uuid,
    ) -> Result<AnnouncementItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.publish_announcement(admin_id, announcement_id);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let mut announcement =
            PostgresAnnouncementRepository::find_announcement(pool, announcement_id)
                .await
                .map_err(|_| ForumError::Internal)?
                .ok_or(ForumError::NotFound)?;
        AnnouncementService::publish(&mut announcement, OffsetDateTime::now_utc())?;
        let rows_affected =
            PostgresAnnouncementRepository::update_announcement_status(pool, &announcement)
                .await
                .map_err(|_| ForumError::Internal)?;
        if rows_affected == 0 {
            return Err(ForumError::NotFound);
        }
        let actions = announcement_published_actions(&announcement);
        PostgresIntegrationRepository::insert_actions(pool, &actions)
            .await
            .map_err(|_| ForumError::Internal)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            "announcement.publish",
            "announcement",
            announcement.announcement_id,
            &announcement_audit_snapshot(&announcement),
        )
        .await
        .map_err(|_| ForumError::Internal)?;

        Ok(announcement)
    }

    pub async fn push_announcement(
        &self,
        admin_id: Uuid,
        announcement_id: Uuid,
    ) -> Result<AnnouncementItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.push_announcement(admin_id, announcement_id);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let announcement = PostgresAnnouncementRepository::find_announcement(pool, announcement_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if announcement.status != AnnouncementStatus::Published {
            return Err(ForumError::Validation("只有已发布公告可以推送".to_string()));
        }

        let recipient_ids =
            PostgresAnnouncementRepository::announcement_recipient_ids(pool, &announcement)
                .await
                .map_err(|_| ForumError::Internal)?;
        for recipient_id in recipient_ids {
            postgres_insert_notification(
                pool,
                recipient_id,
                Some(admin_id),
                NotificationType::Announcement,
                announcement.title.clone(),
                AnnouncementService::notification_body(&announcement.content),
            )
            .await?;
        }
        let actions = announcement_published_actions(&announcement);
        PostgresIntegrationRepository::insert_actions(pool, &actions)
            .await
            .map_err(|_| ForumError::Internal)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            "announcement.push",
            "announcement",
            announcement.announcement_id,
            &announcement_audit_snapshot(&announcement),
        )
        .await
        .map_err(|_| ForumError::Internal)?;

        Ok(announcement)
    }

    pub async fn withdraw_announcement(
        &self,
        admin_id: Uuid,
        announcement_id: Uuid,
    ) -> Result<AnnouncementItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.withdraw_announcement(admin_id, announcement_id);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let mut announcement =
            PostgresAnnouncementRepository::find_announcement(pool, announcement_id)
                .await
                .map_err(|_| ForumError::Internal)?
                .ok_or(ForumError::NotFound)?;
        AnnouncementService::withdraw(&mut announcement, OffsetDateTime::now_utc())?;
        let rows_affected =
            PostgresAnnouncementRepository::update_announcement_status(pool, &announcement)
                .await
                .map_err(|_| ForumError::Internal)?;
        if rows_affected == 0 {
            return Err(ForumError::NotFound);
        }
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            "announcement.withdraw",
            "announcement",
            announcement.announcement_id,
            &announcement_audit_snapshot(&announcement),
        )
        .await
        .map_err(|_| ForumError::Internal)?;

        Ok(announcement)
    }

    pub async fn public_announcements(&self) -> Vec<AnnouncementItem> {
        let Some(pool) = &self.db else {
            return self.forum.public_announcements();
        };

        PostgresAnnouncementRepository::public_announcements(pool, OffsetDateTime::now_utc())
            .await
            .unwrap_or_default()
    }

    pub async fn mark_announcement_read(
        &self,
        user_id: Uuid,
        announcement_id: Uuid,
    ) -> Result<AnnouncementReadState, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.mark_announcement_read(user_id, announcement_id);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, user_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }
        let announcement = PostgresAnnouncementRepository::find_announcement(pool, announcement_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if !announcement.is_public_at(OffsetDateTime::now_utc()) {
            return Err(ForumError::NotFound);
        }

        PostgresAnnouncementRepository::mark_read(pool, announcement_id, user_id)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn public_categories(&self) -> Vec<CategoryItem> {
        let Some(pool) = &self.db else {
            return self.forum.public_categories();
        };

        PostgresTaxonomyRepository::public_categories(pool)
            .await
            .unwrap_or_default()
    }

    pub async fn public_tags(&self) -> Vec<TagItem> {
        let Some(pool) = &self.db else {
            return self.forum.public_tags();
        };

        PostgresTaxonomyRepository::public_tags(pool)
            .await
            .unwrap_or_default()
    }

    pub async fn list_admin_categories(
        &self,
        admin_id: Uuid,
    ) -> Result<Vec<CategoryItem>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.list_admin_categories(admin_id);
        };

        postgres_admin_user(pool, admin_id).await?;
        PostgresTaxonomyRepository::admin_categories(pool)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn create_category(
        &self,
        admin_id: Uuid,
        request: CreateCategoryRequest,
    ) -> Result<CategoryItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.create_category(admin_id, request);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        if PostgresTaxonomyRepository::enabled_category_name_exists(pool, &request.name, None)
            .await
            .map_err(|_| ForumError::Internal)?
        {
            return Err(ForumError::Conflict("分类名称已存在".to_string()));
        }
        let category = TaxonomyService::build_category(Uuid::new_v4(), request)?;
        let category = PostgresTaxonomyRepository::insert_category(pool, &category)
            .await
            .map_err(map_taxonomy_db_error)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            "category.create",
            "category",
            category.category_id,
            &category_audit_snapshot(&category),
        )
        .await
        .map_err(|_| ForumError::Internal)?;
        Ok(category)
    }

    pub async fn update_category(
        &self,
        admin_id: Uuid,
        category_id: Uuid,
        request: UpdateCategoryRequest,
    ) -> Result<CategoryItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.update_category(admin_id, category_id, request);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let audit_action = if request.enabled == Some(false) {
            "category.disable"
        } else {
            "category.update"
        };
        if let Some(name) = &request.name {
            if PostgresTaxonomyRepository::enabled_category_name_exists(
                pool,
                name,
                Some(category_id),
            )
            .await
            .map_err(|_| ForumError::Internal)?
            {
                return Err(ForumError::Conflict("分类名称已存在".to_string()));
            }
        }
        let mut category = PostgresTaxonomyRepository::find_category(pool, category_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        TaxonomyService::apply_category_update(&mut category, request)?;
        let category = PostgresTaxonomyRepository::update_category(pool, &category)
            .await
            .map_err(map_taxonomy_db_error)?
            .ok_or(ForumError::NotFound)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            audit_action,
            "category",
            category.category_id,
            &category_audit_snapshot(&category),
        )
        .await
        .map_err(|_| ForumError::Internal)?;
        Ok(category)
    }

    pub async fn disable_category(
        &self,
        admin_id: Uuid,
        category_id: Uuid,
    ) -> Result<CategoryItem, ForumError> {
        self.update_category(
            admin_id,
            category_id,
            UpdateCategoryRequest {
                name: None,
                color: None,
                sort_order: None,
                enabled: Some(false),
            },
        )
        .await
    }

    pub async fn list_admin_tags(&self, admin_id: Uuid) -> Result<Vec<TagItem>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.list_admin_tags(admin_id);
        };

        postgres_admin_user(pool, admin_id).await?;
        PostgresTaxonomyRepository::admin_tags(pool)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn create_tag(
        &self,
        admin_id: Uuid,
        request: CreateTagRequest,
    ) -> Result<TagItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.create_tag(admin_id, request);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        if PostgresTaxonomyRepository::enabled_tag_name_exists(pool, &request.name, None)
            .await
            .map_err(|_| ForumError::Internal)?
        {
            return Err(ForumError::Conflict("标签名称已存在".to_string()));
        }
        let tag = TaxonomyService::build_tag(Uuid::new_v4(), request)?;
        let tag = PostgresTaxonomyRepository::insert_tag(pool, &tag)
            .await
            .map_err(map_taxonomy_db_error)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            "tag.create",
            "tag",
            tag.tag_id,
            &tag_audit_snapshot(&tag),
        )
        .await
        .map_err(|_| ForumError::Internal)?;
        Ok(tag)
    }

    pub async fn update_tag(
        &self,
        admin_id: Uuid,
        tag_id: Uuid,
        request: UpdateTagRequest,
    ) -> Result<TagItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.update_tag(admin_id, tag_id, request);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let audit_action = if request.enabled == Some(false) && request.use_count == Some(0) {
            "tag.delete"
        } else {
            "tag.update"
        };
        if let Some(name) = &request.name {
            if PostgresTaxonomyRepository::enabled_tag_name_exists(pool, name, Some(tag_id))
                .await
                .map_err(|_| ForumError::Internal)?
            {
                return Err(ForumError::Conflict("标签名称已存在".to_string()));
            }
        }
        let mut tag = PostgresTaxonomyRepository::find_tag(pool, tag_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        TaxonomyService::apply_tag_update(&mut tag, request)?;
        let tag = PostgresTaxonomyRepository::update_tag(pool, &tag)
            .await
            .map_err(map_taxonomy_db_error)?
            .ok_or(ForumError::NotFound)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            audit_action,
            "tag",
            tag.tag_id,
            &tag_audit_snapshot(&tag),
        )
        .await
        .map_err(|_| ForumError::Internal)?;
        Ok(tag)
    }

    pub async fn merge_tag(
        &self,
        admin_id: Uuid,
        source_tag_id: Uuid,
        request: MergeTagRequest,
    ) -> Result<TagItem, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.merge_tag(admin_id, source_tag_id, request);
        };

        TaxonomyService::validate_tag_merge(source_tag_id, request.target_tag_id)?;
        let admin = postgres_admin_user(pool, admin_id).await?;
        let source = PostgresTaxonomyRepository::find_tag(pool, source_tag_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        let target = PostgresTaxonomyRepository::find_tag(pool, request.target_tag_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        let tag = PostgresTaxonomyRepository::merge_tag(pool, &source, &target)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            "tag.merge",
            "tag",
            tag.tag_id,
            &format!(
                "source_tag_id={},source_name={},target={}",
                source.tag_id,
                source.name,
                tag_audit_snapshot(&tag)
            ),
        )
        .await
        .map_err(|_| ForumError::Internal)?;
        Ok(tag)
    }

    pub async fn delete_tag(&self, admin_id: Uuid, tag_id: Uuid) -> Result<TagItem, ForumError> {
        self.update_tag(
            admin_id,
            tag_id,
            UpdateTagRequest {
                name: None,
                sort_order: None,
                enabled: Some(false),
                use_count: Some(0),
            },
        )
        .await
    }

    pub async fn admin_users(&self, admin_id: Uuid) -> Result<Vec<AdminUserRow>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.admin_users(admin_id);
        };

        postgres_admin_user(pool, admin_id).await?;
        PostgresUserAdminRepository::list_users(pool)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn disable_user(
        &self,
        admin_id: Uuid,
        user_id: Uuid,
        context: AuditContext,
    ) -> Result<AdminUserRow, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.disable_user(admin_id, user_id, context);
        };

        UserAdminService::ensure_not_self_disable(admin_id, user_id)?;
        let admin = postgres_admin_user(pool, admin_id).await?;
        PostgresUserAdminRepository::set_user_disabled(
            pool,
            &admin.session_user(),
            user_id,
            true,
            context,
        )
        .await
        .map_err(|_| ForumError::Internal)?
        .ok_or(ForumError::NotFound)
    }

    pub async fn enable_user(
        &self,
        admin_id: Uuid,
        user_id: Uuid,
        context: AuditContext,
    ) -> Result<AdminUserRow, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.enable_user(admin_id, user_id, context);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        PostgresUserAdminRepository::set_user_disabled(
            pool,
            &admin.session_user(),
            user_id,
            false,
            context,
        )
        .await
        .map_err(|_| ForumError::Internal)?
        .ok_or(ForumError::NotFound)
    }

    pub async fn update_user_roles(
        &self,
        admin_id: Uuid,
        user_id: Uuid,
        request: UpdateUserRolesRequest,
    ) -> Result<AdminUserRow, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.update_user_roles(admin_id, user_id, request);
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let roles = UserAdminService::normalize_roles(request.roles.clone())?;
        PostgresUserAdminRepository::update_user_roles(
            pool,
            &admin.session_user(),
            user_id,
            roles,
            request.context,
        )
        .await
        .map_err(|_| ForumError::Internal)?
        .ok_or(ForumError::NotFound)
    }

    pub async fn audit_logs(&self, admin_id: Uuid) -> Result<Vec<AuditLogEntry>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.audit_logs(admin_id);
        };

        postgres_admin_user(pool, admin_id).await?;
        PostgresUserAdminRepository::list_audit_logs(pool)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn list_roles(&self, admin_id: Uuid) -> Result<Vec<Role>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.list_roles(admin_id);
        };

        postgres_admin_user(pool, admin_id).await?;
        PostgresRbacRepository::ensure_seed_data(pool)
            .await
            .map_err(|_| ForumError::Internal)?;
        PostgresRbacRepository::list_roles(pool)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn list_permissions(&self, admin_id: Uuid) -> Result<Vec<Permission>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.list_permissions(admin_id);
        };

        postgres_admin_user(pool, admin_id).await?;
        PostgresRbacRepository::ensure_seed_data(pool)
            .await
            .map_err(|_| ForumError::Internal)?;
        Ok(admin_permissions())
    }

    pub async fn create_role(
        &self,
        admin_id: Uuid,
        request: CreateRoleRequest,
    ) -> Result<Role, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.create_role(admin_id, request);
        };

        let role =
            RbacService::build_role(&request.code, &request.name, &request.permission_codes)?;
        let admin = postgres_admin_user(pool, admin_id).await?;
        PostgresRbacRepository::ensure_seed_data(pool)
            .await
            .map_err(|_| ForumError::Internal)?;
        PostgresRbacRepository::create_role(pool, &admin.session_user(), role, request.context)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or_else(|| ForumError::Conflict("角色已存在".to_string()))
    }

    pub async fn update_role(
        &self,
        admin_id: Uuid,
        role_code: &str,
        request: UpdateRoleRequest,
    ) -> Result<Role, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.update_role(admin_id, role_code, request);
        };

        let code = RbacService::normalize_role_code(role_code)?;
        let admin = postgres_admin_user(pool, admin_id).await?;
        PostgresRbacRepository::ensure_seed_data(pool)
            .await
            .map_err(|_| ForumError::Internal)?;
        let before = PostgresRbacRepository::find_role(pool, &code)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        let mut after = before.clone();
        RbacService::apply_role_update(&mut after, request.clone())?;
        PostgresRbacRepository::update_role(
            pool,
            &admin.session_user(),
            before,
            after,
            request.context,
        )
        .await
        .map_err(|_| ForumError::Internal)?
        .ok_or(ForumError::NotFound)
    }

    pub async fn delete_role(
        &self,
        admin_id: Uuid,
        role_code: &str,
        context: AuditContext,
    ) -> Result<Role, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.delete_role(admin_id, role_code, context);
        };

        let code = RbacService::normalize_role_code(role_code)?;
        RbacService::ensure_deletable_role(&code)?;
        let admin = postgres_admin_user(pool, admin_id).await?;
        PostgresRbacRepository::ensure_seed_data(pool)
            .await
            .map_err(|_| ForumError::Internal)?;
        if PostgresRbacRepository::role_has_users(pool, &code)
            .await
            .map_err(|_| ForumError::Internal)?
        {
            return Err(ForumError::Conflict("角色已分配给用户".to_string()));
        }
        let role = PostgresRbacRepository::find_role(pool, &code)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        PostgresRbacRepository::delete_role(pool, &admin.session_user(), role, context)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)
    }

    pub async fn admin_posts(&self, admin_id: Uuid) -> Result<Vec<ModerationPostRow>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.admin_posts(admin_id);
        };

        postgres_admin_user(pool, admin_id).await?;
        PostgresModerationRepository::list_posts(pool)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn take_down_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_status(admin_id, post_id, PostStatus::Offline)
            .await
    }

    pub async fn restore_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_status(admin_id, post_id, PostStatus::Published)
            .await
    }

    pub async fn delete_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_status(admin_id, post_id, PostStatus::Deleted)
            .await
    }

    pub async fn pin_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_pin(admin_id, post_id, true).await
    }

    pub async fn unpin_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_pin(admin_id, post_id, false).await
    }

    pub async fn recommend_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_recommend(admin_id, post_id, true).await
    }

    pub async fn unrecommend_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_recommend(admin_id, post_id, false).await
    }

    pub async fn lock_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_lock(admin_id, post_id, true).await
    }

    pub async fn unlock_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_lock(admin_id, post_id, false).await
    }

    pub async fn admin_comments(
        &self,
        admin_id: Uuid,
    ) -> Result<Vec<ModerationCommentRow>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.admin_comments(admin_id);
        };

        postgres_admin_user(pool, admin_id).await?;
        PostgresModerationRepository::list_comments(pool)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn delete_comment(
        &self,
        admin_id: Uuid,
        comment_id: Uuid,
    ) -> Result<ModerationCommentAction, ForumError> {
        self.set_comment_deleted(admin_id, comment_id, true).await
    }

    pub async fn recover_comment(
        &self,
        admin_id: Uuid,
        comment_id: Uuid,
    ) -> Result<ModerationCommentAction, ForumError> {
        self.set_comment_deleted(admin_id, comment_id, false).await
    }

    async fn set_post_status(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
        status: PostStatus,
    ) -> Result<ModerationPostAction, ForumError> {
        let Some(pool) = &self.db else {
            return match status {
                PostStatus::Offline => self.forum.take_down_post(admin_id, post_id),
                PostStatus::Published => self.forum.restore_post(admin_id, post_id),
                PostStatus::Deleted => self.forum.delete_post(admin_id, post_id),
                PostStatus::Draft => Err(ForumError::Validation("不能设置为草稿".to_string())),
            };
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let action = PostgresModerationRepository::set_post_status(pool, post_id, &status)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            post_status_audit_action(&status),
            "post",
            post_id,
            &post_moderation_audit_snapshot(&action),
        )
        .await
        .map_err(|_| ForumError::Internal)?;
        Ok(action)
    }

    async fn set_post_pin(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
        pinned: bool,
    ) -> Result<ModerationPostAction, ForumError> {
        let Some(pool) = &self.db else {
            return if pinned {
                self.forum.pin_post(admin_id, post_id)
            } else {
                self.forum.unpin_post(admin_id, post_id)
            };
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let current = PostgresModerationRepository::find_post_action(pool, post_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if current.status == PostStatus::Deleted {
            return Err(ForumError::Conflict("已删除帖子不能置顶".to_string()));
        }
        let action = PostgresModerationRepository::set_post_pin(pool, post_id, pinned)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            if pinned { "post.pin" } else { "post.unpin" },
            "post",
            post_id,
            &post_moderation_audit_snapshot(&action),
        )
        .await
        .map_err(|_| ForumError::Internal)?;
        Ok(action)
    }

    async fn set_post_recommend(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
        recommended: bool,
    ) -> Result<ModerationPostAction, ForumError> {
        let Some(pool) = &self.db else {
            return if recommended {
                self.forum.recommend_post(admin_id, post_id)
            } else {
                self.forum.unrecommend_post(admin_id, post_id)
            };
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let current = PostgresModerationRepository::find_post_action(pool, post_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if current.status == PostStatus::Deleted {
            return Err(ForumError::Conflict("已删除帖子不能推荐".to_string()));
        }
        let action = PostgresModerationRepository::set_post_recommend(pool, post_id, recommended)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            if recommended {
                "post.recommend"
            } else {
                "post.unrecommend"
            },
            "post",
            post_id,
            &post_moderation_audit_snapshot(&action),
        )
        .await
        .map_err(|_| ForumError::Internal)?;
        Ok(action)
    }

    async fn set_post_lock(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
        locked: bool,
    ) -> Result<ModerationPostAction, ForumError> {
        let Some(pool) = &self.db else {
            return if locked {
                self.forum.lock_post(admin_id, post_id)
            } else {
                self.forum.unlock_post(admin_id, post_id)
            };
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let current = PostgresModerationRepository::find_post_action(pool, post_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        if current.status == PostStatus::Deleted {
            return Err(ForumError::Conflict("已删除帖子不能锁定".to_string()));
        }
        let action = PostgresModerationRepository::set_post_lock(pool, post_id, locked)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            if locked { "post.lock" } else { "post.unlock" },
            "post",
            post_id,
            &post_moderation_audit_snapshot(&action),
        )
        .await
        .map_err(|_| ForumError::Internal)?;
        Ok(action)
    }

    async fn set_comment_deleted(
        &self,
        admin_id: Uuid,
        comment_id: Uuid,
        deleted: bool,
    ) -> Result<ModerationCommentAction, ForumError> {
        let Some(pool) = &self.db else {
            return if deleted {
                self.forum.delete_comment(admin_id, comment_id)
            } else {
                self.forum.recover_comment(admin_id, comment_id)
            };
        };

        let admin = postgres_admin_user(pool, admin_id).await?;
        let action = PostgresModerationRepository::set_comment_deleted(pool, comment_id, deleted)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)?;
        PostgresAdminAuditRepository::insert_audit_log(
            pool,
            admin.user_id,
            if deleted {
                "comment.delete"
            } else {
                "comment.recover"
            },
            "comment",
            comment_id,
            &comment_moderation_audit_snapshot(&action),
        )
        .await
        .map_err(|_| ForumError::Internal)?;
        Ok(action)
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        request: UpdateProfileRequest,
    ) -> Result<UserProfile, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.update_profile(user_id, request);
        };

        let profile = UserSettingsService::normalize_profile(request)?;
        PostgresUserSettingsRepository::update_profile(
            pool,
            user_id,
            &profile.nickname,
            &profile.bio,
        )
        .await
        .map_err(|_| ForumError::Internal)?
        .ok_or(ForumError::Unauthorized)
    }

    pub async fn update_avatar(
        &self,
        user_id: Uuid,
        request: UpdateAvatarRequest,
    ) -> Result<UserProfile, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.update_avatar(user_id, request);
        };

        let avatar_url = UserSettingsService::normalize_avatar(request)?;
        PostgresUserSettingsRepository::update_avatar(pool, user_id, &avatar_url)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> Result<(), ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.change_password(user_id, request);
        };

        let user = PostgresAuthRepository::find_user_by_id(pool, user_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if user.is_disabled() {
            return Err(ForumError::Forbidden);
        }
        let new_password =
            UserSettingsService::validate_password_change(&user.password_hash, request)?;
        let new_password_hash = AuthService::hash_password(&new_password);
        let rows_affected =
            PostgresUserSettingsRepository::update_password(pool, user_id, &new_password_hash)
                .await
                .map_err(|_| ForumError::Internal)?;
        if rows_affected == 0 {
            return Err(ForumError::Unauthorized);
        }
        Ok(())
    }

    pub async fn user_space(
        &self,
        profile_user_id: Uuid,
        viewer_user_id: Option<Uuid>,
    ) -> Result<UserSpace, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.user_space(profile_user_id, viewer_user_id);
        };

        PostgresUserSettingsRepository::user_space(pool, profile_user_id, viewer_user_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::NotFound)
    }

    pub async fn notification_center(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationCenter, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.notification_center(user_id);
        };

        PostgresNotificationRepository::notification_center(pool, user_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)
    }

    pub async fn mark_notification_read(
        &self,
        user_id: Uuid,
        notification_id: Uuid,
    ) -> Result<NotificationCenter, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.mark_notification_read(user_id, notification_id);
        };

        let affected =
            PostgresNotificationRepository::mark_notification_read(pool, user_id, notification_id)
                .await
                .map_err(|_| ForumError::Internal)?;
        if affected == 0 {
            return Err(ForumError::NotFound);
        }
        self.notification_center(user_id).await
    }

    pub async fn mark_all_notifications_read(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationCenter, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.mark_all_notifications_read(user_id);
        };

        PostgresNotificationRepository::mark_all_read(pool, user_id)
            .await
            .map_err(|_| ForumError::Internal)?;
        self.notification_center(user_id).await
    }

    pub async fn home_page(
        &self,
        query: HomeQuery,
        current_user_id: Option<Uuid>,
    ) -> Result<HomePageData, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.home_page(query, current_user_id);
        };

        let mut home = dense_workbench_home(query, current_user_id.is_some());
        if home.requires_login {
            return Ok(home);
        }

        home.topics = PostgresPostRepository::list_homepage_summaries(
            pool,
            &home.query,
            current_user_id,
            postgres_demo_home_enabled(),
        )
        .await
        .map_err(|_| ForumError::Internal)?
        .into_iter()
        .map(crate::domain::home::home_topic_from_post_summary)
        .collect();
        let total =
            PostgresPostRepository::count_homepage_summaries(pool, &home.query, current_user_id)
                .await
                .map_err(|_| ForumError::Internal)? as usize;
        let total = postgres_demo_home_total(&home.query).unwrap_or(total);
        let total_pages = total.div_ceil(home.query.page_size).max(1);
        let shown_start = if total == 0 {
            0
        } else {
            (home.query.page.saturating_sub(1)) * home.query.page_size + 1
        };
        let shown_end = (shown_start.saturating_sub(1) + home.topics.len()).min(total);
        home.pagination = crate::domain::home::HomePagination {
            page: home.query.page,
            page_size: home.query.page_size,
            total,
            total_pages,
            label: format!("显示 {shown_start}-{shown_end} / {total} 个主题"),
        };

        let runtime_config = RuntimeConfig::from_env();
        if runtime_config.home_sidebar_cache_enabled {
            if let Ok(cache) = RedisHomeCacheRepository::from_url(
                &runtime_config.redis_url,
                runtime_config.home_sidebar_cache_ttl_seconds,
            ) {
                if let Ok(Some(snapshot)) = cache.try_read_sidebar().await {
                    snapshot.apply_to_home(&mut home);
                    return Ok(home);
                }
            }
        }

        home.categories = PostgresHomeRepository::list_homepage_categories(pool)
            .await
            .map_err(|_| ForumError::Internal)?;
        home.hot_tags = PostgresHomeRepository::list_hot_tags(pool, 8)
            .await
            .map_err(|_| ForumError::Internal)?;
        home.announcements = PostgresHomeRepository::list_announcements(pool, 3)
            .await
            .map_err(|_| ForumError::Internal)?;
        home.active_authors = PostgresHomeRepository::list_active_authors(pool, 5)
            .await
            .map_err(|_| ForumError::Internal)?;

        if runtime_config.home_sidebar_cache_enabled {
            if let Ok(cache) = RedisHomeCacheRepository::from_url(
                &runtime_config.redis_url,
                runtime_config.home_sidebar_cache_ttl_seconds,
            ) {
                let _ = cache
                    .write_sidebar(&HomeSidebarSnapshot::from_home(&home))
                    .await;
            }
        }

        Ok(home)
    }

    pub async fn search(&self, query: SearchQuery) -> Result<SearchResultPage, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.search(query);
        };

        let runtime_config = RuntimeConfig::from_env();
        if runtime_config.search_backend == "elasticsearch" {
            let repository = ElasticsearchSearchRepository::from_url(
                &runtime_config.elasticsearch_url,
                runtime_config.elasticsearch_search_index,
            )
            .map_err(|_| ForumError::Internal)?;
            return repository
                .search_posts(query)
                .await
                .map_err(|_| ForumError::Internal);
        }

        PostgresSearchRepository::search_posts(pool, query)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn upload_file(
        &self,
        uploader_id: Uuid,
        request: FileUploadRequest,
    ) -> Result<FileAsset, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.upload_file(uploader_id, request);
        };

        request.validate().map_err(ForumError::Validation)?;
        let uploader = PostgresAuthRepository::find_user_by_id(pool, uploader_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if uploader.is_disabled() {
            return Err(ForumError::Forbidden);
        }
        if let Some(existing) = PostgresFileRepository::find_by_hash(pool, &request.content_hash)
            .await
            .map_err(|_| ForumError::Internal)?
        {
            return Ok(existing);
        }

        let usage = request.usage.clone();
        let asset = build_file_asset(Uuid::new_v4(), uploader_id, request);
        PostgresFileRepository::insert_asset(pool, &asset, &usage)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn upload_binary_file(
        &self,
        uploader_id: Uuid,
        request: FileBinaryUploadRequest,
    ) -> Result<FileAsset, ForumError> {
        let object = request.to_object_upload().map_err(ForumError::Validation)?;
        let Some(pool) = &self.db else {
            return self.forum.upload_file(uploader_id, object.asset);
        };

        let uploader = PostgresAuthRepository::find_user_by_id(pool, uploader_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .ok_or(ForumError::Unauthorized)?;
        if uploader.is_disabled() {
            return Err(ForumError::Forbidden);
        }
        if let Some(existing) =
            PostgresFileRepository::find_by_hash(pool, &object.asset.content_hash)
                .await
                .map_err(|_| ForumError::Internal)?
        {
            return Ok(existing);
        }

        let config = RustfsObjectStoreConfig::from_runtime_config(&RuntimeConfig::from_env());
        let object_store = RustfsObjectStore::from_config(config)
            .await
            .map_err(|_| ForumError::Internal)?;
        object_store
            .put_object(object.clone())
            .await
            .map_err(|_| ForumError::Internal)?;

        let usage = object.asset.usage.clone();
        let asset = build_file_asset(Uuid::new_v4(), uploader_id, object.asset);
        PostgresFileRepository::insert_asset(pool, &asset, &usage)
            .await
            .map_err(|_| ForumError::Internal)
    }

    pub async fn admin_dashboard(&self, user_id: Uuid) -> Result<AdminDashboard, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.admin_dashboard(user_id);
        };

        postgres_admin_user(pool, user_id).await?;
        let users = self.admin_users(user_id).await?;
        let posts = self.admin_posts(user_id).await?;
        let comments = self.admin_comments(user_id).await?;
        let categories = self.list_admin_categories(user_id).await?;
        let tags = self.list_admin_tags(user_id).await?;
        let announcements = self.list_admin_announcements(user_id).await?;
        let reports = self.list_reports(user_id).await?;
        let audit_logs = self.audit_logs(user_id).await?;
        let roles = self.list_roles(user_id).await?;
        let stats_summary = PostgresAdminStatsRepository::load_summary(pool)
            .await
            .map_err(|_| ForumError::Internal)?;
        let hot_post = PostgresAdminStatsRepository::top_hot_post(pool)
            .await
            .map_err(|_| ForumError::Internal)?;
        let hot_tag = PostgresAdminStatsRepository::top_hot_tag(pool)
            .await
            .map_err(|_| ForumError::Internal)?;
        let online_connections = self.forum.total_online_connections()?;

        let mut dashboard = admin_dashboard_demo();
        dashboard.stats = vec![
            admin_stat("用户总数", stats_summary.user_count, "PostgreSQL"),
            admin_stat("今日新增用户", stats_summary.today_user_count, "今日"),
            admin_stat("帖子总数", stats_summary.post_count, "PostgreSQL"),
            admin_stat("今日新增帖子", stats_summary.today_post_count, "今日"),
            admin_stat("评论总数", stats_summary.comment_count, "PostgreSQL"),
            admin_stat("今日新增评论", stats_summary.today_comment_count, "今日"),
            admin_stat("点赞总数", stats_summary.like_count, "帖子 + 评论"),
            admin_stat("当前在线用户数", online_connections as i64, "WebSocket"),
            admin_stat_text(
                "热门帖子",
                hot_post
                    .as_ref()
                    .map(|post| post.title.as_str())
                    .unwrap_or("暂无"),
                hot_post
                    .as_ref()
                    .map(|post| format!("热度 {}", post.hot_score))
                    .unwrap_or_else(|| "无公开帖子".to_string()),
            ),
            admin_stat_text(
                "热门标签",
                hot_tag
                    .as_ref()
                    .map(|tag| tag.name.as_str())
                    .unwrap_or("暂无"),
                hot_tag
                    .as_ref()
                    .map(|tag| format!("{} 篇帖子", tag.use_count))
                    .unwrap_or_else(|| "无公开标签".to_string()),
            ),
        ];
        dashboard.permissions = admin_permissions();
        dashboard.roles = roles;
        dashboard.users = users.into_iter().map(dashboard_user_row).collect();
        dashboard.moderation_posts = posts.into_iter().map(dashboard_post_row).collect();
        dashboard.moderation_comments = comments.into_iter().map(dashboard_comment_row).collect();
        dashboard.categories = categories.into_iter().map(dashboard_category_row).collect();
        dashboard.tags = tags.into_iter().map(dashboard_tag_row).collect();
        dashboard.announcements = announcements
            .into_iter()
            .map(dashboard_announcement_row)
            .collect();
        dashboard.reports = reports.into_iter().map(dashboard_report_row).collect();
        dashboard.audit_entries = audit_logs.into_iter().map(dashboard_audit_entry).collect();
        Ok(dashboard)
    }

    pub async fn connect_notification_socket(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.connect_notification_socket(user_id);
        };

        ensure_postgres_active_user(pool, user_id).await?;
        self.forum.connect_notification_socket_trusted(user_id)
    }

    pub async fn disconnect_notification_socket(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.disconnect_notification_socket(user_id);
        };

        ensure_postgres_active_user(pool, user_id).await?;
        self.forum.disconnect_notification_socket_trusted(user_id)
    }

    pub async fn notification_connection_stats(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.notification_connection_stats(user_id);
        };

        ensure_postgres_active_user(pool, user_id).await?;
        self.forum.notification_connection_stats_trusted(user_id)
    }

    pub async fn pending_notification_pushes(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<NotificationPush>, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.pending_notification_pushes(user_id);
        };

        ensure_postgres_active_user(pool, user_id).await?;
        self.forum.pending_notification_pushes_trusted(user_id)
    }

    pub async fn ack_notification_push(
        &self,
        user_id: Uuid,
        push_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let Some(pool) = &self.db else {
            return self.forum.ack_notification_push(user_id, push_id);
        };

        ensure_postgres_active_user(pool, user_id).await?;
        self.forum.ack_notification_push_trusted(user_id, push_id)
    }
}

#[cfg(feature = "ssr")]
fn map_user_insert_error(error: sqlx::Error) -> ForumError {
    if error
        .as_database_error()
        .is_some_and(|db_error| db_error.is_unique_violation())
    {
        ForumError::Conflict("用户名已存在".to_string())
    } else {
        ForumError::Internal
    }
}

#[cfg(feature = "ssr")]
fn map_taxonomy_db_error(error: sqlx::Error) -> ForumError {
    if error
        .as_database_error()
        .is_some_and(|db_error| db_error.is_unique_violation())
    {
        ForumError::Conflict("名称已存在".to_string())
    } else {
        ForumError::Internal
    }
}

#[cfg(feature = "ssr")]
async fn postgres_admin_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<crate::repositories::auth::UserAuthRow, ForumError> {
    let user = PostgresAuthRepository::find_user_by_id(pool, user_id)
        .await
        .map_err(|_| ForumError::Internal)?
        .ok_or(ForumError::Unauthorized)?;
    if user.is_disabled() || !user.is_admin {
        return Err(ForumError::Forbidden);
    }
    Ok(user)
}

#[cfg(feature = "ssr")]
async fn ensure_postgres_active_user(pool: &PgPool, user_id: Uuid) -> Result<(), ForumError> {
    let user = PostgresAuthRepository::find_user_by_id(pool, user_id)
        .await
        .map_err(|_| ForumError::Internal)?
        .ok_or(ForumError::Unauthorized)?;
    if user.is_disabled() {
        return Err(ForumError::Forbidden);
    }
    Ok(())
}

#[cfg(feature = "ssr")]
async fn postgres_insert_notification(
    pool: &PgPool,
    recipient_id: Uuid,
    actor_id: Option<Uuid>,
    notification_type: NotificationType,
    title: String,
    body: String,
) -> Result<(), ForumError> {
    let notification = Notification {
        notification_id: Uuid::new_v4(),
        recipient_id,
        actor_id,
        notification_type,
        title,
        body,
        read_at: None,
        created_at: OffsetDateTime::now_utc(),
    };
    PostgresNotificationRepository::insert_notification(pool, &notification)
        .await
        .map_err(|_| ForumError::Internal)
}

#[cfg(feature = "ssr")]
async fn postgres_report_target_title(
    pool: &PgPool,
    request: &CreateReportRequest,
) -> Result<Option<String>, ForumError> {
    match request.target_type {
        ReportTargetType::Post => PostgresPostRepository::find_detail(pool, request.target_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .map(|post| Some(post.summary.title))
            .ok_or(ForumError::NotFound),
        ReportTargetType::Comment => PostgresCommentRepository::find_by_id(pool, request.target_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .map(|comment| Some(comment.content.chars().take(40).collect()))
            .ok_or(ForumError::NotFound),
        ReportTargetType::User => PostgresAuthRepository::find_user_by_id(pool, request.target_id)
            .await
            .map_err(|_| ForumError::Internal)?
            .map(|user| Some(user.nickname))
            .ok_or(ForumError::NotFound),
    }
}

#[cfg(feature = "ssr")]
fn admin_stat(label: &str, value: i64, delta: &str) -> AdminStat {
    AdminStat {
        label: label.to_string(),
        value: value.to_string(),
        delta: delta.to_string(),
    }
}

#[cfg(feature = "ssr")]
fn admin_stat_text(label: &str, value: &str, delta: String) -> AdminStat {
    AdminStat {
        label: label.to_string(),
        value: value.to_string(),
        delta,
    }
}

#[cfg(feature = "ssr")]
fn post_status_audit_action(status: &PostStatus) -> &'static str {
    match status {
        PostStatus::Offline => "post.take_down",
        PostStatus::Published => "post.restore",
        PostStatus::Deleted => "post.delete",
        PostStatus::Draft => "post.update",
    }
}

#[cfg(feature = "ssr")]
fn post_moderation_audit_snapshot(action: &ModerationPostAction) -> String {
    format!(
        "post_id={},status={:?},pinned={},recommended={},locked={}",
        action.post_id, action.status, action.pinned, action.recommended, action.locked
    )
}

#[cfg(feature = "ssr")]
fn comment_moderation_audit_snapshot(action: &ModerationCommentAction) -> String {
    format!(
        "comment_id={},post_id={},deleted={}",
        action.comment_id, action.post_id, action.deleted
    )
}

#[cfg(feature = "ssr")]
fn report_audit_snapshot(report: &ReportItem) -> String {
    format!(
        "report_id={},target_type={:?},target_id={},status={:?},handler_id={:?}",
        report.report_id, report.target_type, report.target_id, report.status, report.handler_id
    )
}

#[cfg(feature = "ssr")]
fn announcement_audit_snapshot(announcement: &AnnouncementItem) -> String {
    format!(
        "announcement_id={},title={},status={:?},pinned={}",
        announcement.announcement_id, announcement.title, announcement.status, announcement.pinned
    )
}

#[cfg(feature = "ssr")]
fn category_audit_snapshot(category: &CategoryItem) -> String {
    format!(
        "category_id={},name={},color={},sort_order={},enabled={}",
        category.category_id, category.name, category.color, category.sort_order, category.enabled
    )
}

#[cfg(feature = "ssr")]
fn tag_audit_snapshot(tag: &TagItem) -> String {
    format!(
        "tag_id={},name={},sort_order={},enabled={},use_count={}",
        tag.tag_id, tag.name, tag.sort_order, tag.enabled, tag.use_count
    )
}

#[cfg(feature = "ssr")]
fn dashboard_user_row(row: AdminUserRow) -> DashboardUserRow {
    DashboardUserRow {
        user_id: row.user_id,
        username: row.username,
        nickname: row.nickname,
        roles: row.roles,
        status: if row.disabled { "已禁用" } else { "正常" }.to_string(),
        actions: if row.disabled {
            vec!["解禁用户".to_string(), "调整角色".to_string()]
        } else {
            vec!["调整角色".to_string(), "禁用用户".to_string()]
        },
    }
}

#[cfg(feature = "ssr")]
fn dashboard_post_row(row: ModerationPostRow) -> AdminPostRow {
    AdminPostRow {
        post_id: row.post_id,
        title: row.title,
        author: row.author_name,
        category: row.category_name.unwrap_or_else(|| "未分类".to_string()),
        status: post_status_label(&row.status).to_string(),
        recommended: row.recommended,
        locked: row.locked,
        actions: match row.status {
            PostStatus::Published => vec![
                "下架".to_string(),
                if row.recommended {
                    "取消推荐"
                } else {
                    "推荐"
                }
                .to_string(),
                if row.pinned { "取消置顶" } else { "置顶" }.to_string(),
                if row.locked { "解锁" } else { "锁定" }.to_string(),
                "删除".to_string(),
            ],
            PostStatus::Offline => vec!["恢复".to_string(), "查看".to_string(), "删除".to_string()],
            PostStatus::Draft => vec!["查看".to_string(), "删除".to_string()],
            PostStatus::Deleted => vec!["恢复".to_string(), "查看".to_string()],
        },
    }
}

#[cfg(feature = "ssr")]
fn dashboard_comment_row(row: ModerationCommentRow) -> AdminCommentRow {
    AdminCommentRow {
        comment_id: row.comment_id,
        post_id: row.post_id,
        post_title: row.post_title,
        author: row.author_name,
        content: row.content,
        status: if row.deleted { "已删除" } else { "正常" }.to_string(),
        actions: if row.deleted {
            vec!["恢复评论".to_string(), "查看帖子".to_string()]
        } else {
            vec!["删除评论".to_string(), "查看帖子".to_string()]
        },
    }
}

#[cfg(feature = "ssr")]
fn dashboard_category_row(row: CategoryItem) -> AdminCategoryRow {
    AdminCategoryRow {
        category_id: row.category_id,
        name: row.name,
        color: row.color,
        sort_order: row.sort_order,
        post_count: row.post_count,
        status: if row.enabled { "启用" } else { "停用" }.to_string(),
        actions: vec![
            "编辑".to_string(),
            "调整排序".to_string(),
            if row.enabled { "停用" } else { "启用" }.to_string(),
        ],
    }
}

#[cfg(feature = "ssr")]
fn dashboard_tag_row(row: TagItem) -> AdminTagRow {
    AdminTagRow {
        tag_id: row.tag_id,
        name: row.name,
        sort_order: row.sort_order,
        use_count: row.use_count,
        status: if row.enabled { "启用" } else { "停用" }.to_string(),
        actions: vec![
            "编辑".to_string(),
            "合并标签".to_string(),
            if row.enabled { "禁用" } else { "启用" }.to_string(),
        ],
    }
}

#[cfg(feature = "ssr")]
fn dashboard_announcement_row(row: AnnouncementItem) -> AdminAnnouncementRow {
    AdminAnnouncementRow {
        announcement_id: row.announcement_id,
        title: row.title,
        content: row.content,
        announcement_type: row.announcement_type,
        pinned: row.pinned,
        effective_at: row.effective_at,
        expires_at: row.expires_at,
        audience: announcement_audience_label(&row.audience),
        status: announcement_status_label(&row.status).to_string(),
        actions: match row.status {
            AnnouncementStatus::Draft => vec!["发布公告".to_string(), "编辑".to_string()],
            AnnouncementStatus::Published => {
                vec!["下线公告".to_string(), "推送公告".to_string()]
            }
            AnnouncementStatus::Withdrawn => vec!["重新发布".to_string(), "编辑".to_string()],
        },
    }
}

#[cfg(feature = "ssr")]
fn dashboard_report_row(row: ReportItem) -> AdminReportRow {
    AdminReportRow {
        report_id: row.report_id,
        target: row
            .target_title
            .unwrap_or_else(|| row.target_id.hyphenated().to_string()),
        target_type: report_target_type_label(&row.target_type).to_string(),
        reason: row.reason,
        reporter: row.reporter_name,
        status: report_status_label(&row.status).to_string(),
        actions: match row.status {
            ReportStatus::Pending => vec![
                "标记已处理".to_string(),
                "驳回".to_string(),
                "删除违规内容".to_string(),
            ],
            ReportStatus::Handled | ReportStatus::Rejected => vec!["查看详情".to_string()],
        },
    }
}

#[cfg(feature = "ssr")]
fn dashboard_audit_entry(row: AuditLogEntry) -> AuditEntry {
    AuditEntry {
        actor: row.actor_name,
        action: row.action,
        target: row.target_label,
        ip: row.ip.unwrap_or_else(|| "unknown".to_string()),
        user_agent: row.user_agent.unwrap_or_else(|| "unknown".to_string()),
        time_label: row.created_at.to_string(),
    }
}

#[cfg(feature = "ssr")]
fn post_status_label(status: &PostStatus) -> &'static str {
    match status {
        PostStatus::Draft => "草稿",
        PostStatus::Published => "已发布",
        PostStatus::Offline => "已下架",
        PostStatus::Deleted => "已删除",
    }
}

#[cfg(feature = "ssr")]
fn announcement_audience_label(audience: &AnnouncementAudience) -> String {
    match audience {
        AnnouncementAudience::AllUsers => "全体用户".to_string(),
        AnnouncementAudience::UserIds(user_ids) => format!("指定用户({})", user_ids.len()),
    }
}

#[cfg(feature = "ssr")]
fn announcement_status_label(status: &AnnouncementStatus) -> &'static str {
    match status {
        AnnouncementStatus::Draft => "草稿",
        AnnouncementStatus::Published => "已发布",
        AnnouncementStatus::Withdrawn => "已撤回",
    }
}

#[cfg(feature = "ssr")]
fn report_target_type_label(target_type: &ReportTargetType) -> &'static str {
    match target_type {
        ReportTargetType::Post => "帖子",
        ReportTargetType::Comment => "评论",
        ReportTargetType::User => "用户",
    }
}

#[cfg(feature = "ssr")]
fn report_status_label(status: &ReportStatus) -> &'static str {
    match status {
        ReportStatus::Pending => "待处理",
        ReportStatus::Handled => "已处理",
        ReportStatus::Rejected => "已驳回",
    }
}

#[cfg(feature = "ssr")]
fn postgres_demo_home_total(query: &HomeQuery) -> Option<usize> {
    let default_query = HomeQuery::default().normalized();
    (postgres_demo_home_enabled() && query == &default_query).then_some(342)
}

#[cfg(feature = "ssr")]
fn postgres_demo_home_enabled() -> bool {
    std::env::var("POST_DEMO_SEED_HOME")
        .map(|value| value == "true" || value == "1")
        .unwrap_or(false)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub database_url: String,
    pub redis_url: String,
    pub home_sidebar_cache_enabled: bool,
    pub home_sidebar_cache_ttl_seconds: usize,
    pub nats_url: String,
    pub rustfs_bucket: String,
    pub elasticsearch_url: String,
    pub elasticsearch_search_index: String,
    pub search_backend: String,
    pub integration_worker_enabled: bool,
    pub integration_worker_batch_size: i64,
    pub integration_worker_max_attempts: i32,
    pub integration_worker_interval_millis: u64,
}

impl RuntimeConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://post:post@localhost:5433/post".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6380".to_string()),
            home_sidebar_cache_enabled: std::env::var("HOME_SIDEBAR_CACHE_ENABLED")
                .map(|value| value == "true" || value == "1")
                .unwrap_or(false),
            home_sidebar_cache_ttl_seconds: std::env::var("HOME_SIDEBAR_CACHE_TTL_SECONDS")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(60),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            rustfs_bucket: std::env::var("RUSTFS_BUCKET")
                .unwrap_or_else(|_| "post-assets".to_string()),
            elasticsearch_url: std::env::var("ELASTICSEARCH_URL")
                .unwrap_or_else(|_| "http://localhost:9200".to_string()),
            elasticsearch_search_index: std::env::var("ELASTICSEARCH_SEARCH_INDEX")
                .unwrap_or_else(|_| "posts".to_string()),
            search_backend: std::env::var("SEARCH_BACKEND")
                .unwrap_or_else(|_| "postgres".to_string())
                .trim()
                .to_lowercase(),
            integration_worker_enabled: std::env::var("INTEGRATION_WORKER_ENABLED")
                .map(|value| value == "true" || value == "1")
                .unwrap_or(false),
            integration_worker_batch_size: std::env::var("INTEGRATION_WORKER_BATCH_SIZE")
                .ok()
                .and_then(|value| value.parse::<i64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(50),
            integration_worker_max_attempts: std::env::var("INTEGRATION_WORKER_MAX_ATTEMPTS")
                .ok()
                .and_then(|value| value.parse::<i32>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(3),
            integration_worker_interval_millis: std::env::var("INTEGRATION_WORKER_INTERVAL_MILLIS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(1_000),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ForumStore {
    inner: Arc<RwLock<ForumData>>,
}

#[derive(Debug)]
struct ForumData {
    users: HashMap<Uuid, SessionUser>,
    user_bios: HashMap<Uuid, String>,
    user_registered_at: HashMap<Uuid, OffsetDateTime>,
    user_passwords: HashMap<Uuid, String>,
    sessions: HashMap<Uuid, Session>,
    posts: HashMap<Uuid, PostDetail>,
    post_order: Vec<Uuid>,
    comments: HashMap<Uuid, Vec<CommentNode>>,
    files: HashMap<Uuid, FileAsset>,
    notifications: HashMap<Uuid, Vec<Notification>>,
    notification_connections: HashMap<Uuid, usize>,
    pending_notification_pushes: HashMap<Uuid, Vec<NotificationPush>>,
    integration_actions: Vec<IntegrationAction>,
    announcements: HashMap<Uuid, AnnouncementItem>,
    announcement_reads: HashSet<(Uuid, Uuid)>,
    post_reads: HashSet<(Uuid, Uuid)>,
    reports: HashMap<Uuid, ReportItem>,
    categories: HashMap<Uuid, CategoryItem>,
    tags: HashMap<Uuid, TagItem>,
    liked_posts: HashSet<(Uuid, Uuid)>,
    liked_comments: HashSet<(Uuid, Uuid)>,
    favorited_posts: HashSet<(Uuid, Uuid)>,
    follows: HashSet<(Uuid, Uuid)>,
    pinned_posts: HashSet<Uuid>,
    recommended_posts: HashSet<Uuid>,
    disabled_users: HashSet<Uuid>,
    roles: HashMap<String, Role>,
    user_roles: HashMap<Uuid, Vec<String>>,
    audit_logs: Vec<AuditLogEntry>,
    next_id: u128,
}

impl ForumStore {
    pub fn seeded() -> Self {
        let author_id = Uuid::from_u128(1);
        let post_id = Uuid::from_u128(2);
        let now = OffsetDateTime::now_utc();

        let author = SessionUser {
            user_id: author_id,
            username: "mah".to_string(),
            nickname: "mah".to_string(),
            avatar_url: None,
            is_admin: true,
        };

        let summary = PostSummary {
            post_id,
            title: "Rust 异步任务的边界设计".to_string(),
            summary: "从论坛系统的通知链路拆分 Tokio 任务、事务和事件投递。".to_string(),
            author_id,
            author_name: author.nickname.clone(),
            author_avatar_url: author.avatar_url.clone(),
            category_name: Some("Rust".to_string()),
            tags: vec!["Leptos".to_string(), "SQLx".to_string()],
            view_count: 128,
            comment_count: 0,
            like_count: 19,
            favorite_count: 8,
            published_at: Some(now),
            last_reply_author_name: None,
            last_reply_author_avatar_url: None,
            last_reply_at: None,
            pinned: false,
            locked: false,
            read_by_me: false,
        };

        let detail = PostDetail {
            summary,
            markdown: "## Rust 异步任务的边界\n\n把通知、搜索索引和计数更新从请求链路中拆出去。"
                .to_string(),
            sanitized_html: render_markdown_safe(
                "## Rust 异步任务的边界\n\n把通知、搜索索引和计数更新从请求链路中拆出去。",
            ),
            status: PostStatus::Published,
            locked: false,
            liked_by_me: false,
            favorited_by_me: false,
            following_author: false,
        };

        let mut users = HashMap::new();
        users.insert(author_id, author);
        let mut user_bios = HashMap::new();
        user_bios.insert(
            author_id,
            "Post Forum 管理员，关注 Leptos、Axum 与 SQLx。".to_string(),
        );
        let mut user_registered_at = HashMap::new();
        user_registered_at.insert(author_id, now);
        let mut user_passwords = HashMap::new();
        user_passwords.insert(author_id, AuthService::hash_password("password"));
        let roles = seed_roles();
        let mut user_roles = HashMap::new();
        user_roles.insert(author_id, vec!["admin".to_string()]);

        let mut posts = HashMap::new();
        posts.insert(post_id, detail);
        let categories = seed_category_items();
        let tags = seed_tag_items();

        Self {
            inner: Arc::new(RwLock::new(ForumData {
                users,
                user_bios,
                user_registered_at,
                user_passwords,
                sessions: HashMap::new(),
                posts,
                post_order: vec![post_id],
                comments: HashMap::new(),
                files: HashMap::new(),
                notifications: HashMap::new(),
                notification_connections: HashMap::new(),
                pending_notification_pushes: HashMap::new(),
                integration_actions: Vec::new(),
                announcements: HashMap::new(),
                announcement_reads: HashSet::new(),
                post_reads: HashSet::new(),
                reports: HashMap::new(),
                categories,
                tags,
                liked_posts: HashSet::new(),
                liked_comments: HashSet::new(),
                favorited_posts: HashSet::new(),
                follows: HashSet::new(),
                pinned_posts: HashSet::new(),
                recommended_posts: HashSet::new(),
                disabled_users: HashSet::new(),
                roles,
                user_roles,
                audit_logs: Vec::new(),
                next_id: 3,
            })),
        }
    }

    pub fn demo_user(&self) -> SessionUser {
        self.inner
            .read()
            .expect("forum store lock")
            .users
            .values()
            .next()
            .expect("seed user")
            .clone()
    }

    pub fn login(&self, username: &str, password: &str) -> Result<Session, ForumError> {
        let login = AuthService::normalize_login(username, password)?;

        let mut data = self.write_data()?;
        let user = data
            .users
            .values()
            .find(|user| user.username == login.username)
            .cloned()
            .unwrap_or_else(|| {
                let user_id = next_uuid(&mut data);
                let user = AuthService::build_login_user(user_id, &login.username);
                data.users.insert(user_id, user.clone());
                data.user_roles.insert(user_id, vec!["member".to_string()]);
                data.user_bios.insert(user_id, String::new());
                data.user_registered_at
                    .insert(user_id, OffsetDateTime::now_utc());
                data.user_passwords
                    .insert(user_id, AuthService::hash_password(&login.password));
                user
            });
        if data.disabled_users.contains(&user.user_id) {
            return Err(ForumError::Forbidden);
        }
        if let Some(stored_password) = data.user_passwords.get(&user.user_id) {
            AuthService::validate_password_match(stored_password, &login.password)?;
        }

        let session_id = next_uuid(&mut data);
        let session = AuthService::build_session(session_id, user, OffsetDateTime::now_utc());
        data.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    pub fn register(&self, request: RegisterRequest) -> Result<Session, ForumError> {
        let registration = AuthService::normalize_registration(request)?;

        let mut data = self.write_data()?;
        if data
            .users
            .values()
            .any(|user| user.username == registration.username)
        {
            return Err(ForumError::Conflict("用户名已存在".to_string()));
        }

        let user_id = next_uuid(&mut data);
        let password = AuthService::hash_password(&registration.password);
        let user = AuthService::build_registered_user(user_id, registration);
        data.users.insert(user_id, user.clone());
        data.user_roles.insert(user_id, vec!["member".to_string()]);
        data.user_bios.insert(user_id, String::new());
        data.user_registered_at
            .insert(user_id, OffsetDateTime::now_utc());
        data.user_passwords.insert(user_id, password);

        let session_id = next_uuid(&mut data);
        let session = AuthService::build_session(session_id, user, OffsetDateTime::now_utc());
        data.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    pub fn current_session(&self, session_id: Uuid) -> Result<Session, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        let session = data
            .sessions
            .get(&session_id)
            .cloned()
            .ok_or(ForumError::Unauthorized)?;
        AuthService::validate_session_active(session.expires_at, OffsetDateTime::now_utc())?;
        if data.disabled_users.contains(&session.user.user_id) {
            return Err(ForumError::Forbidden);
        }
        Ok(session)
    }

    pub fn logout(&self, session_id: Uuid) -> Result<Session, ForumError> {
        let mut data = self.write_data()?;
        data.sessions
            .remove(&session_id)
            .ok_or(ForumError::Unauthorized)
    }

    pub fn list_posts(&self) -> Vec<PostSummary> {
        let data = self.inner.read().expect("forum store lock");
        data.post_order
            .iter()
            .filter_map(|post_id| data.posts.get(post_id))
            .filter(|detail| detail.status == PostStatus::Published)
            .map(|detail| detail.summary.clone())
            .collect()
    }

    pub fn integration_actions(&self) -> Vec<IntegrationAction> {
        self.inner
            .read()
            .expect("forum store lock")
            .integration_actions
            .clone()
    }

    pub fn home_page(
        &self,
        query: HomeQuery,
        current_user_id: Option<Uuid>,
    ) -> Result<HomePageData, ForumError> {
        if let Some(user_id) = current_user_id {
            let data = self.inner.read().map_err(|_| ForumError::Internal)?;
            if !data.users.contains_key(&user_id) {
                return Err(ForumError::Unauthorized);
            }
        }

        let mut home = dense_workbench_home(query, current_user_id.is_some());
        home.categories = self
            .public_categories()
            .into_iter()
            .map(|category| HomeCategory {
                name: category.name,
                count: category.post_count,
                color: category.color,
            })
            .collect();
        home.hot_tags = self
            .public_tags()
            .into_iter()
            .take(8)
            .map(|tag| HomeTag {
                name: tag.name,
                count: tag.use_count,
            })
            .collect();
        let announcements = self.public_announcements();
        if !announcements.is_empty() {
            home.announcements = announcements
                .into_iter()
                .take(3)
                .map(home_announcement)
                .collect();
        }
        Ok(home)
    }

    pub fn search(&self, query: SearchQuery) -> Result<SearchResultPage, ForumError> {
        Ok(search_dense_workbench(query))
    }

    pub fn admin_dashboard(&self, user_id: Uuid) -> Result<AdminDashboard, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        let user = data.users.get(&user_id).ok_or(ForumError::Unauthorized)?;
        if !user.is_admin {
            return Err(ForumError::Forbidden);
        }

        let mut dashboard = admin_dashboard_demo();
        dashboard.roles = sorted_roles(data.roles.values().cloned());
        Ok(dashboard)
    }

    pub fn create_announcement(
        &self,
        admin_id: Uuid,
        request: CreateAnnouncementRequest,
    ) -> Result<AnnouncementItem, ForumError> {
        let mut data = self.write_data()?;
        let admin = ensure_admin(&data, admin_id)?.clone();
        let now = OffsetDateTime::now_utc();
        let announcement_id = next_uuid(&mut data);
        let announcement = AnnouncementService::build_draft(
            announcement_id,
            admin_id,
            &admin.nickname,
            request,
            now,
        )?;
        data.announcements
            .insert(announcement_id, announcement.clone());
        Ok(announcement)
    }

    pub fn update_announcement(
        &self,
        admin_id: Uuid,
        announcement_id: Uuid,
        request: UpdateAnnouncementRequest,
    ) -> Result<AnnouncementItem, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        let announcement = data
            .announcements
            .get_mut(&announcement_id)
            .ok_or(ForumError::NotFound)?;
        AnnouncementService::apply_update(announcement, request, OffsetDateTime::now_utc())?;
        Ok(announcement.clone())
    }

    pub fn list_admin_announcements(
        &self,
        admin_id: Uuid,
    ) -> Result<Vec<AnnouncementItem>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        ensure_admin(&data, admin_id)?;
        Ok(sorted_announcements(data.announcements.values().cloned()))
    }

    pub fn publish_announcement(
        &self,
        admin_id: Uuid,
        announcement_id: Uuid,
    ) -> Result<AnnouncementItem, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        let now = OffsetDateTime::now_utc();
        let recipients = {
            let announcement = data
                .announcements
                .get(&announcement_id)
                .ok_or(ForumError::NotFound)?;
            announcement_recipients(&data, announcement)
        };

        let announcement = data
            .announcements
            .get_mut(&announcement_id)
            .ok_or(ForumError::NotFound)?;
        AnnouncementService::publish(announcement, now)?;
        let published = announcement.clone();

        for recipient_id in recipients {
            push_notification(
                &mut data,
                recipient_id,
                Some(admin_id),
                NotificationType::Announcement,
                published.title.clone(),
                AnnouncementService::notification_body(&published.content),
            );
        }
        data.integration_actions
            .extend(announcement_published_actions(&published));
        Ok(published)
    }

    pub fn push_announcement(
        &self,
        admin_id: Uuid,
        announcement_id: Uuid,
    ) -> Result<AnnouncementItem, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        let announcement = data
            .announcements
            .get(&announcement_id)
            .ok_or(ForumError::NotFound)?
            .clone();
        if announcement.status != AnnouncementStatus::Published {
            return Err(ForumError::Validation("只有已发布公告可以推送".to_string()));
        }

        for recipient_id in announcement_recipients(&data, &announcement) {
            push_notification(
                &mut data,
                recipient_id,
                Some(admin_id),
                NotificationType::Announcement,
                announcement.title.clone(),
                AnnouncementService::notification_body(&announcement.content),
            );
        }
        data.integration_actions
            .extend(announcement_published_actions(&announcement));
        Ok(announcement)
    }

    pub fn withdraw_announcement(
        &self,
        admin_id: Uuid,
        announcement_id: Uuid,
    ) -> Result<AnnouncementItem, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        let announcement = data
            .announcements
            .get_mut(&announcement_id)
            .ok_or(ForumError::NotFound)?;
        let now = OffsetDateTime::now_utc();
        AnnouncementService::withdraw(announcement, now)?;
        Ok(announcement.clone())
    }

    pub fn public_announcements(&self) -> Vec<AnnouncementItem> {
        let Ok(data) = self.inner.read() else {
            return Vec::new();
        };
        let now = OffsetDateTime::now_utc();
        sorted_announcements(
            data.announcements
                .values()
                .filter(|announcement| announcement.is_public_at(now))
                .cloned(),
        )
    }

    pub fn mark_announcement_read(
        &self,
        user_id: Uuid,
        announcement_id: Uuid,
    ) -> Result<AnnouncementReadState, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }
        let announcement = data
            .announcements
            .get(&announcement_id)
            .ok_or(ForumError::NotFound)?;
        if !announcement.is_public_at(OffsetDateTime::now_utc()) {
            return Err(ForumError::NotFound);
        }
        data.announcement_reads.insert((user_id, announcement_id));
        Ok(AnnouncementReadState {
            announcement_id,
            user_id,
            read: true,
        })
    }

    pub fn public_categories(&self) -> Vec<CategoryItem> {
        let Ok(data) = self.inner.read() else {
            return Vec::new();
        };
        sorted_categories(
            data.categories
                .values()
                .filter(|item| item.enabled)
                .cloned(),
        )
    }

    pub fn public_tags(&self) -> Vec<TagItem> {
        let Ok(data) = self.inner.read() else {
            return Vec::new();
        };
        sorted_tags(data.tags.values().filter(|item| item.enabled).cloned())
    }

    pub fn list_admin_categories(&self, admin_id: Uuid) -> Result<Vec<CategoryItem>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        ensure_admin(&data, admin_id)?;
        Ok(sorted_categories(data.categories.values().cloned()))
    }

    pub fn list_admin_tags(&self, admin_id: Uuid) -> Result<Vec<TagItem>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        ensure_admin(&data, admin_id)?;
        Ok(sorted_tags(data.tags.values().cloned()))
    }

    pub fn admin_users(&self, admin_id: Uuid) -> Result<Vec<AdminUserRow>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        ensure_admin(&data, admin_id)?;
        Ok(admin_user_rows(&data))
    }

    pub fn list_roles(&self, admin_id: Uuid) -> Result<Vec<Role>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        ensure_admin(&data, admin_id)?;
        Ok(sorted_roles(data.roles.values().cloned()))
    }

    pub fn list_permissions(&self, admin_id: Uuid) -> Result<Vec<Permission>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        ensure_admin(&data, admin_id)?;
        Ok(admin_permissions())
    }

    pub fn create_role(
        &self,
        admin_id: Uuid,
        request: CreateRoleRequest,
    ) -> Result<Role, ForumError> {
        let role =
            RbacService::build_role(&request.code, &request.name, &request.permission_codes)?;
        let code = role.code.clone();
        let mut data = self.write_data()?;
        let actor = ensure_admin(&data, admin_id)?.clone();
        if data.roles.contains_key(&code) {
            return Err(ForumError::Conflict("角色已存在".to_string()));
        }
        data.roles.insert(code.clone(), role.clone());
        push_audit_log(
            &mut data,
            &actor,
            "role.create",
            "role",
            Uuid::nil(),
            code,
            None,
            Some(role_snapshot(&role)),
            request.context,
        );
        Ok(role)
    }

    pub fn update_role(
        &self,
        admin_id: Uuid,
        role_code: &str,
        request: UpdateRoleRequest,
    ) -> Result<Role, ForumError> {
        let code = RbacService::normalize_role_code(role_code)?;
        let mut data = self.write_data()?;
        let actor = ensure_admin(&data, admin_id)?.clone();
        let before = data.roles.get(&code).map(role_snapshot);
        let role = data.roles.get_mut(&code).ok_or(ForumError::NotFound)?;
        RbacService::apply_role_update(role, request.clone())?;
        let updated = role.clone();
        push_audit_log(
            &mut data,
            &actor,
            "role.update",
            "role",
            Uuid::nil(),
            code,
            before,
            Some(role_snapshot(&updated)),
            request.context,
        );
        Ok(updated)
    }

    pub fn delete_role(
        &self,
        admin_id: Uuid,
        role_code: &str,
        context: AuditContext,
    ) -> Result<Role, ForumError> {
        let code = RbacService::normalize_role_code(role_code)?;
        RbacService::ensure_deletable_role(&code)?;
        let mut data = self.write_data()?;
        let actor = ensure_admin(&data, admin_id)?.clone();
        if data
            .user_roles
            .values()
            .any(|roles| roles.iter().any(|role| role == &code))
        {
            return Err(ForumError::Conflict("角色已分配给用户".to_string()));
        }
        let role = data.roles.remove(&code).ok_or(ForumError::NotFound)?;
        push_audit_log(
            &mut data,
            &actor,
            "role.delete",
            "role",
            Uuid::nil(),
            code,
            Some(role_snapshot(&role)),
            None,
            context,
        );
        Ok(role)
    }

    pub fn disable_user(
        &self,
        admin_id: Uuid,
        user_id: Uuid,
        context: AuditContext,
    ) -> Result<AdminUserRow, ForumError> {
        UserAdminService::ensure_not_self_disable(admin_id, user_id)?;

        let mut data = self.write_data()?;
        let actor = ensure_admin(&data, admin_id)?.clone();
        let target = data
            .users
            .get(&user_id)
            .cloned()
            .ok_or(ForumError::NotFound)?;
        let before = user_audit_snapshot(&data, user_id);
        data.disabled_users.insert(user_id);
        let after = user_audit_snapshot(&data, user_id);
        push_audit_log(
            &mut data,
            &actor,
            "user.disable",
            "user",
            user_id,
            target.nickname,
            before,
            after,
            context,
        );
        admin_user_row(&data, user_id).ok_or(ForumError::NotFound)
    }

    pub fn enable_user(
        &self,
        admin_id: Uuid,
        user_id: Uuid,
        context: AuditContext,
    ) -> Result<AdminUserRow, ForumError> {
        let mut data = self.write_data()?;
        let actor = ensure_admin(&data, admin_id)?.clone();
        let target = data
            .users
            .get(&user_id)
            .cloned()
            .ok_or(ForumError::NotFound)?;
        let before = user_audit_snapshot(&data, user_id);
        data.disabled_users.remove(&user_id);
        let after = user_audit_snapshot(&data, user_id);
        push_audit_log(
            &mut data,
            &actor,
            "user.enable",
            "user",
            user_id,
            target.nickname,
            before,
            after,
            context,
        );
        admin_user_row(&data, user_id).ok_or(ForumError::NotFound)
    }

    pub fn update_user_roles(
        &self,
        admin_id: Uuid,
        user_id: Uuid,
        request: UpdateUserRolesRequest,
    ) -> Result<AdminUserRow, ForumError> {
        let mut data = self.write_data()?;
        let actor = ensure_admin(&data, admin_id)?.clone();
        let target = data
            .users
            .get(&user_id)
            .cloned()
            .ok_or(ForumError::NotFound)?;
        let before = user_audit_snapshot(&data, user_id);
        let roles = UserAdminService::normalize_roles(request.roles.clone())?;
        if roles.iter().any(|role| !data.roles.contains_key(role)) {
            return Err(ForumError::Validation("角色不存在".to_string()));
        }
        data.user_roles.insert(user_id, roles);
        let after = user_audit_snapshot(&data, user_id);
        push_audit_log(
            &mut data,
            &actor,
            "user.roles.update",
            "user",
            user_id,
            target.nickname,
            before,
            after,
            request.context,
        );
        admin_user_row(&data, user_id).ok_or(ForumError::NotFound)
    }

    pub fn audit_logs(&self, admin_id: Uuid) -> Result<Vec<AuditLogEntry>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        ensure_admin(&data, admin_id)?;
        let mut logs = data.audit_logs.clone();
        logs.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(logs)
    }

    pub fn admin_posts(&self, admin_id: Uuid) -> Result<Vec<ModerationPostRow>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        ensure_admin(&data, admin_id)?;
        Ok(data
            .post_order
            .iter()
            .filter_map(|post_id| data.posts.get(post_id))
            .map(|post| {
                ModerationService::post_row(
                    post,
                    data.pinned_posts.contains(&post.summary.post_id),
                    data.recommended_posts.contains(&post.summary.post_id),
                )
            })
            .collect())
    }

    pub fn take_down_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_status(admin_id, post_id, PostStatus::Offline)
    }

    pub fn restore_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_status(admin_id, post_id, PostStatus::Published)
    }

    pub fn delete_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_status(admin_id, post_id, PostStatus::Deleted)
    }

    pub fn pin_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_pin(admin_id, post_id, true)
    }

    pub fn unpin_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_pin(admin_id, post_id, false)
    }

    pub fn recommend_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_recommend(admin_id, post_id, true)
    }

    pub fn unrecommend_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_recommend(admin_id, post_id, false)
    }

    pub fn lock_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_lock(admin_id, post_id, true)
    }

    pub fn unlock_post(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
    ) -> Result<ModerationPostAction, ForumError> {
        self.set_post_lock(admin_id, post_id, false)
    }

    pub fn admin_comments(&self, admin_id: Uuid) -> Result<Vec<ModerationCommentRow>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        ensure_admin(&data, admin_id)?;
        Ok(data
            .comments
            .iter()
            .flat_map(|(post_id, comments)| {
                let post_title = data
                    .posts
                    .get(post_id)
                    .map(|post| post.summary.title.clone())
                    .unwrap_or_else(|| "已删除帖子".to_string());
                ModerationService::flatten_comment_rows(*post_id, &post_title, comments)
            })
            .collect())
    }

    pub fn delete_comment(
        &self,
        admin_id: Uuid,
        comment_id: Uuid,
    ) -> Result<ModerationCommentAction, ForumError> {
        self.set_comment_deleted(admin_id, comment_id, true)
    }

    pub fn recover_comment(
        &self,
        admin_id: Uuid,
        comment_id: Uuid,
    ) -> Result<ModerationCommentAction, ForumError> {
        self.set_comment_deleted(admin_id, comment_id, false)
    }

    pub fn create_category(
        &self,
        admin_id: Uuid,
        request: CreateCategoryRequest,
    ) -> Result<CategoryItem, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        ensure_category_name_unique(&data, &request.name, None)?;
        let category_id = next_uuid(&mut data);
        let category = TaxonomyService::build_category(category_id, request)?;
        data.categories.insert(category_id, category.clone());
        Ok(category)
    }

    pub fn update_category(
        &self,
        admin_id: Uuid,
        category_id: Uuid,
        request: UpdateCategoryRequest,
    ) -> Result<CategoryItem, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        if let Some(name) = &request.name {
            ensure_category_name_unique(&data, name, Some(category_id))?;
        }
        let category = data
            .categories
            .get_mut(&category_id)
            .ok_or(ForumError::NotFound)?;
        TaxonomyService::apply_category_update(category, request)?;
        Ok(category.clone())
    }

    pub fn disable_category(
        &self,
        admin_id: Uuid,
        category_id: Uuid,
    ) -> Result<CategoryItem, ForumError> {
        self.update_category(
            admin_id,
            category_id,
            UpdateCategoryRequest {
                name: None,
                color: None,
                sort_order: None,
                enabled: Some(false),
            },
        )
    }

    pub fn create_tag(
        &self,
        admin_id: Uuid,
        request: CreateTagRequest,
    ) -> Result<TagItem, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        ensure_tag_name_unique(&data, &request.name, None)?;
        let tag_id = next_uuid(&mut data);
        let tag = TaxonomyService::build_tag(tag_id, request)?;
        data.tags.insert(tag_id, tag.clone());
        Ok(tag)
    }

    pub fn update_tag(
        &self,
        admin_id: Uuid,
        tag_id: Uuid,
        request: UpdateTagRequest,
    ) -> Result<TagItem, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        if let Some(name) = &request.name {
            ensure_tag_name_unique(&data, name, Some(tag_id))?;
        }
        let tag = data.tags.get_mut(&tag_id).ok_or(ForumError::NotFound)?;
        TaxonomyService::apply_tag_update(tag, request)?;
        Ok(tag.clone())
    }

    pub fn merge_tag(
        &self,
        admin_id: Uuid,
        source_tag_id: Uuid,
        request: MergeTagRequest,
    ) -> Result<TagItem, ForumError> {
        TaxonomyService::validate_tag_merge(source_tag_id, request.target_tag_id)?;

        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        let source_count = data
            .tags
            .get(&source_tag_id)
            .ok_or(ForumError::NotFound)?
            .use_count;
        let target = data
            .tags
            .get_mut(&request.target_tag_id)
            .ok_or(ForumError::NotFound)?;
        TaxonomyService::apply_target_merge(target, source_count);
        let merged = target.clone();
        if let Some(source) = data.tags.get_mut(&source_tag_id) {
            TaxonomyService::disable_merged_source(source);
        }
        Ok(merged)
    }

    pub fn delete_tag(&self, admin_id: Uuid, tag_id: Uuid) -> Result<TagItem, ForumError> {
        self.update_tag(
            admin_id,
            tag_id,
            UpdateTagRequest {
                name: None,
                sort_order: None,
                enabled: Some(false),
                use_count: Some(0),
            },
        )
    }

    pub fn create_report(
        &self,
        reporter_id: Uuid,
        request: CreateReportRequest,
    ) -> Result<ReportItem, ForumError> {
        let mut data = self.write_data()?;
        let reporter = data
            .users
            .get(&reporter_id)
            .cloned()
            .ok_or(ForumError::Unauthorized)?;
        let target_title = report_target_title(&data, &request)?;
        let report_id = next_uuid(&mut data);
        let report = ReportService::build_report(
            report_id,
            reporter_id,
            &reporter.nickname,
            target_title,
            request,
            OffsetDateTime::now_utc(),
        )?;
        data.reports.insert(report_id, report.clone());
        Ok(report)
    }

    pub fn list_reports(&self, admin_id: Uuid) -> Result<Vec<ReportItem>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        ensure_admin(&data, admin_id)?;

        let mut reports = data.reports.values().cloned().collect::<Vec<_>>();
        reports.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(reports)
    }

    pub fn handle_report(
        &self,
        admin_id: Uuid,
        report_id: Uuid,
        request: HandleReportRequest,
    ) -> Result<ReportItem, ForumError> {
        let mut data = self.write_data()?;
        let admin = ensure_admin(&data, admin_id)?.clone();
        let report = data
            .reports
            .get_mut(&report_id)
            .ok_or(ForumError::NotFound)?;

        ReportService::apply_handle(
            report,
            admin_id,
            &admin.nickname,
            request,
            OffsetDateTime::now_utc(),
        )?;
        Ok(report.clone())
    }

    pub fn upload_file(
        &self,
        uploader_id: Uuid,
        request: FileUploadRequest,
    ) -> Result<FileAsset, ForumError> {
        request.validate().map_err(ForumError::Validation)?;

        let mut data = self.write_data()?;
        if !data.users.contains_key(&uploader_id) {
            return Err(ForumError::Unauthorized);
        }

        if let Some(existing) = data
            .files
            .values()
            .find(|asset| asset.file_hash == request.content_hash)
            .cloned()
        {
            return Ok(existing);
        }

        let file_id = next_uuid(&mut data);
        let asset = build_file_asset(file_id, uploader_id, request);
        data.files.insert(file_id, asset.clone());
        Ok(asset)
    }

    pub fn upload_binary_file(
        &self,
        uploader_id: Uuid,
        request: FileBinaryUploadRequest,
    ) -> Result<FileAsset, ForumError> {
        let object = request.to_object_upload().map_err(ForumError::Validation)?;
        self.upload_file(uploader_id, object.asset)
    }

    pub fn update_profile(
        &self,
        user_id: Uuid,
        request: UpdateProfileRequest,
    ) -> Result<UserProfile, ForumError> {
        let profile = UserSettingsService::normalize_profile(request)?;

        let mut data = self.write_data()?;
        if data.disabled_users.contains(&user_id) {
            return Err(ForumError::Forbidden);
        }
        {
            let user = data
                .users
                .get_mut(&user_id)
                .ok_or(ForumError::Unauthorized)?;
            user.nickname = profile.nickname.clone();
        }
        data.user_bios.insert(user_id, profile.bio);
        for post in data.posts.values_mut() {
            if post.summary.author_id == user_id {
                post.summary.author_name = profile.nickname.clone();
            }
        }
        for comments in data.comments.values_mut() {
            update_comment_author_name(comments, user_id, &profile.nickname);
        }
        let user = data.users.get(&user_id).ok_or(ForumError::Unauthorized)?;
        Ok(user_profile(&data, user))
    }

    pub fn update_avatar(
        &self,
        user_id: Uuid,
        request: UpdateAvatarRequest,
    ) -> Result<UserProfile, ForumError> {
        let avatar_url = UserSettingsService::normalize_avatar(request)?;

        let mut data = self.write_data()?;
        if data.disabled_users.contains(&user_id) {
            return Err(ForumError::Forbidden);
        }
        {
            let user = data
                .users
                .get_mut(&user_id)
                .ok_or(ForumError::Unauthorized)?;
            user.avatar_url = Some(avatar_url.clone());
        }
        for post in data.posts.values_mut() {
            if post.summary.author_id == user_id {
                post.summary.author_avatar_url = Some(avatar_url.clone());
            }
        }
        let user = data.users.get(&user_id).ok_or(ForumError::Unauthorized)?;
        Ok(user_profile(&data, user))
    }

    pub fn change_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> Result<(), ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }
        if data.disabled_users.contains(&user_id) {
            return Err(ForumError::Forbidden);
        }
        let stored_password = data
            .user_passwords
            .get(&user_id)
            .ok_or(ForumError::Unauthorized)?;
        let new_password = UserSettingsService::validate_password_change(stored_password, request)?;
        data.user_passwords
            .insert(user_id, AuthService::hash_password(&new_password));
        Ok(())
    }

    pub fn user_space(
        &self,
        profile_user_id: Uuid,
        viewer_user_id: Option<Uuid>,
    ) -> Result<UserSpace, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        let user = data
            .users
            .get(&profile_user_id)
            .cloned()
            .ok_or(ForumError::NotFound)?;

        let profile = user_profile(&data, &user);
        let published_posts = data
            .posts
            .values()
            .filter(|post| {
                post.summary.author_id == profile_user_id && post.status == PostStatus::Published
            })
            .map(|post| post.summary.clone())
            .collect::<Vec<_>>();
        let draft_posts = data
            .posts
            .values()
            .filter(|post| {
                post.summary.author_id == profile_user_id && post.status == PostStatus::Draft
            })
            .map(|post| post.summary.clone())
            .collect::<Vec<_>>();
        let comments = data
            .comments
            .iter()
            .flat_map(|(post_id, comments)| {
                let post_title = data
                    .posts
                    .get(post_id)
                    .map(|post| post.summary.title.clone())
                    .unwrap_or_else(|| "已删除帖子".to_string());
                flatten_user_comments(*post_id, &post_title, comments, profile_user_id)
            })
            .collect::<Vec<_>>();
        let favorite_posts = data
            .favorited_posts
            .iter()
            .filter(|(user_id, _)| *user_id == profile_user_id)
            .filter_map(|(_, post_id)| data.posts.get(post_id))
            .map(|post| post.summary.clone())
            .collect::<Vec<_>>();
        let following = data
            .follows
            .iter()
            .filter(|(follower_id, _)| *follower_id == profile_user_id)
            .filter_map(|(_, followee_id)| data.users.get(followee_id))
            .map(|user| user_profile(&data, user))
            .collect::<Vec<_>>();
        let followers = data
            .follows
            .iter()
            .filter(|(_, followee_id)| *followee_id == profile_user_id)
            .filter_map(|(follower_id, _)| data.users.get(follower_id))
            .map(|user| user_profile(&data, user))
            .collect::<Vec<_>>();
        let received_likes = data
            .posts
            .values()
            .filter(|post| post.summary.author_id == profile_user_id)
            .map(|post| post.summary.like_count)
            .sum();
        let received_favorites = data
            .posts
            .values()
            .filter(|post| post.summary.author_id == profile_user_id)
            .map(|post| post.summary.favorite_count)
            .sum();
        let followed_by_viewer = viewer_user_id
            .is_some_and(|viewer_id| data.follows.contains(&(viewer_id, profile_user_id)));

        Ok(UserSpace {
            profile,
            stats: UserStats {
                following: following.len(),
                followers: followers.len(),
                published_posts: published_posts.len(),
                received_likes,
                received_favorites,
            },
            is_me: viewer_user_id == Some(profile_user_id),
            followed_by_viewer,
            published_posts,
            draft_posts,
            comments,
            favorite_posts,
            following,
            followers,
        })
    }

    pub fn post_detail(&self, post_id: Uuid) -> Result<PostDetail, ForumError> {
        let mut data = self.write_data()?;
        let detail = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
        detail.summary.view_count += 1;
        Ok(detail.clone())
    }

    pub fn post_detail_for_user(
        &self,
        post_id: Uuid,
        current_user_id: Option<Uuid>,
    ) -> Result<PostDetail, ForumError> {
        let mut data = self.write_data()?;
        if let Some(user_id) = current_user_id {
            if !data.users.contains_key(&user_id) {
                return Err(ForumError::Unauthorized);
            }
            if data.disabled_users.contains(&user_id) {
                return Err(ForumError::Forbidden);
            }
        }
        let mut should_mark_read = false;
        let mut detail = {
            let detail = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
            detail.summary.view_count += 1;
            if current_user_id.is_some() && detail.status == PostStatus::Published {
                should_mark_read = true;
                detail.summary.read_by_me = true;
            }
            detail.clone()
        };
        if let Some(user_id) = current_user_id {
            if should_mark_read {
                data.post_reads.insert((user_id, post_id));
            }
        }
        detail.summary.read_by_me = current_user_id
            .map(|user_id| data.post_reads.contains(&(user_id, post_id)))
            .unwrap_or(false);
        Ok(detail)
    }

    pub fn related_posts_for_post(
        &self,
        post_id: Uuid,
        limit: usize,
    ) -> Result<Vec<PostSummary>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        let source = data.posts.get(&post_id).ok_or(ForumError::NotFound)?;
        let source_tags = source
            .summary
            .tags
            .iter()
            .map(|tag| tag.to_lowercase())
            .collect::<HashSet<_>>();
        let source_category = source.summary.category_name.as_deref();
        let mut related = data
            .posts
            .values()
            .filter(|post| post.summary.post_id != post_id)
            .filter(|post| post.status == PostStatus::Published)
            .filter(|post| {
                let shares_tag = post
                    .summary
                    .tags
                    .iter()
                    .any(|tag| source_tags.contains(&tag.to_lowercase()));
                let shares_category = source_category.is_some()
                    && post.summary.category_name.as_deref() == source_category;
                shares_tag || shares_category
            })
            .map(|post| post.summary.clone())
            .collect::<Vec<_>>();
        related.sort_by(|left, right| {
            let left_shared_tags = shared_tag_count(&left.tags, &source_tags);
            let right_shared_tags = shared_tag_count(&right.tags, &source_tags);
            right_shared_tags
                .cmp(&left_shared_tags)
                .then_with(|| right.comment_count.cmp(&left.comment_count))
                .then_with(|| right.view_count.cmp(&left.view_count))
        });
        related.truncate(limit);
        Ok(related)
    }

    pub fn autosave_draft(
        &self,
        author_id: Uuid,
        request: AutosaveDraftRequest,
    ) -> Result<PostDetail, ForumError> {
        let mut data = self.write_data()?;
        let author = data
            .users
            .get(&author_id)
            .cloned()
            .ok_or(ForumError::Unauthorized)?;
        if data.disabled_users.contains(&author_id) {
            return Err(ForumError::Forbidden);
        }

        let target_post_id = request.post_id;
        if let Some(post_id) = target_post_id {
            let detail = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
            if detail.summary.author_id != author_id {
                return Err(ForumError::Forbidden);
            }
            if detail.status != PostStatus::Draft {
                return Err(ForumError::Conflict("只能自动保存草稿".to_string()));
            }

            PostAuthoringService::apply_autosave(detail, request)?;
            return Ok(detail.clone());
        }

        let post_id = next_uuid(&mut data);
        let detail = PostAuthoringService::build_draft(post_id, &author, request)?;

        data.post_order.insert(0, post_id);
        data.posts.insert(post_id, detail.clone());
        Ok(detail)
    }

    pub fn update_post(
        &self,
        author_id: Uuid,
        post_id: Uuid,
        request: UpdatePostRequest,
    ) -> Result<PostDetail, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&author_id) {
            return Err(ForumError::Unauthorized);
        }
        if data.disabled_users.contains(&author_id) {
            return Err(ForumError::Forbidden);
        }

        let detail = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
        if detail.summary.author_id != author_id {
            return Err(ForumError::Forbidden);
        }
        if detail.status == PostStatus::Deleted {
            return Err(ForumError::NotFound);
        }

        PostAuthoringService::apply_update(detail, request, OffsetDateTime::now_utc())?;

        Ok(detail.clone())
    }

    pub fn delete_own_post(
        &self,
        author_id: Uuid,
        post_id: Uuid,
    ) -> Result<PostDetail, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&author_id) {
            return Err(ForumError::Unauthorized);
        }
        if data.disabled_users.contains(&author_id) {
            return Err(ForumError::Forbidden);
        }

        let detail = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
        if detail.summary.author_id != author_id {
            return Err(ForumError::Forbidden);
        }
        detail.status = PostStatus::Deleted;
        Ok(detail.clone())
    }

    pub fn create_post(
        &self,
        author_id: Uuid,
        request: CreatePostRequest,
    ) -> Result<PostDetail, ForumError> {
        let mut data = self.write_data()?;
        let author = data
            .users
            .get(&author_id)
            .cloned()
            .ok_or(ForumError::Unauthorized)?;
        let author_name = author.nickname.clone();
        let post_id = next_uuid(&mut data);
        let publish = request.publish;
        let detail =
            PostAuthoringService::build_post(post_id, &author, request, OffsetDateTime::now_utc())?;
        let title = detail.summary.title.clone();

        data.post_order.insert(0, post_id);
        data.posts.insert(post_id, detail.clone());

        if publish {
            data.integration_actions
                .extend(post_published_actions(&detail));
            for (follower_id, followee_id) in data.follows.clone() {
                if followee_id == author_id && follower_id != author_id {
                    push_notification(
                        &mut data,
                        follower_id,
                        Some(author_id),
                        NotificationType::FollowedUserPosted,
                        format!("{author_name} 发布了新帖子"),
                        title.to_string(),
                    );
                }
            }
        }
        Ok(detail)
    }

    pub fn add_comment(
        &self,
        author_id: Uuid,
        request: CreateCommentRequest,
    ) -> Result<CommentNode, ForumError> {
        let mut data = self.write_data()?;
        let author = data
            .users
            .get(&author_id)
            .cloned()
            .ok_or(ForumError::Unauthorized)?;
        let author_name = author.nickname.clone();
        let post_author_id = data
            .posts
            .get(&request.post_id)
            .ok_or(ForumError::NotFound)?
            .summary
            .author_id;
        let comment_id = next_uuid(&mut data);

        let comment = CommentService::build_comment(
            comment_id,
            request.post_id,
            request.parent_comment_id,
            author_id,
            &author_name,
            post_author_id,
            &request.content,
            OffsetDateTime::now_utc(),
        )?;

        let comments = data.comments.entry(request.post_id).or_default();
        if let Some(parent_id) = request.parent_comment_id {
            append_reply(comments, parent_id, comment.clone())?;
        } else {
            comments.push(comment.clone());
        }

        let post_snapshot = if let Some(post) = data.posts.get_mut(&request.post_id) {
            post.summary.comment_count += 1;
            Some(post.clone())
        } else {
            None
        };

        if author_id != post_author_id {
            push_notification(
                &mut data,
                post_author_id,
                Some(author_id),
                if request.parent_comment_id.is_some() {
                    NotificationType::CommentReplied
                } else {
                    NotificationType::PostCommented
                },
                format!("{author_name} 评论了你的帖子"),
                CommentService::notification_body(&comment.content),
            );
        }
        if let Some(post_snapshot) = post_snapshot {
            data.integration_actions
                .extend(post_comment_changed_actions(
                    &post_snapshot,
                    comment.comment_id,
                ));
        }
        Ok(comment)
    }

    pub fn comments_for_post(&self, post_id: Uuid) -> Result<Vec<CommentNode>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        if !data.posts.contains_key(&post_id) {
            return Err(ForumError::NotFound);
        }
        Ok(data
            .comments
            .get(&post_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(CommentService::mask_deleted)
            .collect())
    }

    pub fn comments_page_for_post(
        &self,
        post_id: Uuid,
        query: CommentPageQuery,
    ) -> Result<CommentPage, ForumError> {
        let query = query.normalized();
        let comments = self.comments_for_post(post_id)?;
        let total = comments.len();
        let start = (query.page.saturating_sub(1)) * query.page_size;
        let page_comments = comments
            .into_iter()
            .skip(start)
            .take(query.page_size)
            .collect();
        Ok(CommentPage {
            comments: page_comments,
            page: query.page,
            page_size: query.page_size,
            total,
            total_pages: total.div_ceil(query.page_size).max(1),
        })
    }

    pub fn toggle_post_like(
        &self,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<ToggleResult, ForumError> {
        self.toggle_post_set(user_id, post_id, ReactionKind::Like)
    }

    pub fn toggle_post_favorite(
        &self,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<ToggleResult, ForumError> {
        self.toggle_post_set(user_id, post_id, ReactionKind::Favorite)
    }

    pub fn delete_own_comment(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<CommentNode, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }
        if data.disabled_users.contains(&user_id) {
            return Err(ForumError::Forbidden);
        }

        for (post_id, comments) in data.comments.iter_mut() {
            if let Some(comment) = find_comment_mut(comments, comment_id) {
                if comment.author_id != user_id {
                    return Err(ForumError::Forbidden);
                }
                let changed = !comment.deleted;
                comment.deleted = true;
                let deleted = CommentService::mask_deleted(comment.clone());
                let post_id = *post_id;
                if changed {
                    if let Some(post) = data.posts.get_mut(&post_id) {
                        post.summary.comment_count = (post.summary.comment_count - 1).max(0);
                    }
                }
                return Ok(deleted);
            }
        }
        Err(ForumError::NotFound)
    }

    pub fn toggle_comment_like(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<ToggleResult, ForumError> {
        let mut data = self.write_data()?;
        let actor_name = data
            .users
            .get(&user_id)
            .map(|user| user.nickname.clone())
            .ok_or(ForumError::Unauthorized)?;
        if data.disabled_users.contains(&user_id) {
            return Err(ForumError::Forbidden);
        }

        let comment_author_id = find_comment_with_post(&data.comments, comment_id)
            .map(|(_, comment)| {
                if comment.deleted {
                    Err(ForumError::Conflict("已删除评论不能点赞".to_string()))
                } else {
                    Ok(comment.author_id)
                }
            })
            .ok_or(ForumError::NotFound)??;

        let key = (user_id, comment_id);
        let active = ReactionService::toggle_pair(&mut data.liked_comments, key);
        let count = data
            .comments
            .values_mut()
            .find_map(|comments| find_comment_mut(comments, comment_id))
            .map(|comment| ReactionService::apply_counter_delta(&mut comment.like_count, active))
            .ok_or(ForumError::NotFound)?;

        if active && comment_author_id != user_id {
            push_notification(
                &mut data,
                comment_author_id,
                Some(user_id),
                NotificationType::CommentLiked,
                format!("{actor_name} 点赞了你的评论"),
                "你的评论获得了新的点赞".to_string(),
            );
        }

        Ok(ToggleResult { active, count })
    }

    pub fn report_comment(
        &self,
        reporter_id: Uuid,
        comment_id: Uuid,
        mut request: CreateReportRequest,
    ) -> Result<ReportItem, ForumError> {
        request.target_type = ReportTargetType::Comment;
        request.target_id = comment_id;
        self.create_report(reporter_id, request)
    }

    pub fn follow_user(
        &self,
        follower_id: Uuid,
        followee_id: Uuid,
    ) -> Result<FollowState, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&follower_id) || !data.users.contains_key(&followee_id) {
            return Err(ForumError::NotFound);
        }

        FollowService::toggle_follow(&mut data.follows, follower_id, followee_id)
    }

    pub fn notification_center(&self, user_id: Uuid) -> Result<NotificationCenter, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        let mut items = data
            .notifications
            .get(&user_id)
            .cloned()
            .unwrap_or_default();
        items.sort_by(|left, right| right.created_at.cmp(&left.created_at));

        Ok(NotificationCenter {
            recipient_id: user_id,
            unread_count: unread_count(&items),
            items,
        })
    }

    pub fn mark_notification_read(
        &self,
        user_id: Uuid,
        notification_id: Uuid,
    ) -> Result<NotificationCenter, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        let notifications = data
            .notifications
            .get_mut(&user_id)
            .ok_or(ForumError::NotFound)?;
        let notification = notifications
            .iter_mut()
            .find(|notification| notification.notification_id == notification_id)
            .ok_or(ForumError::NotFound)?;
        notification.read_at = Some(OffsetDateTime::now_utc());
        drop(data);
        self.notification_center(user_id)
    }

    pub fn mark_all_notifications_read(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationCenter, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        let now = OffsetDateTime::now_utc();
        for notification in data.notifications.entry(user_id).or_default() {
            notification.read_at = Some(now);
        }
        drop(data);
        self.notification_center(user_id)
    }

    pub fn connect_notification_socket(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        *data.notification_connections.entry(user_id).or_default() += 1;
        Ok(notification_connection_stats(&data, user_id))
    }

    pub fn disconnect_notification_socket(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        if let Some(connections) = data.notification_connections.get_mut(&user_id) {
            *connections = connections.saturating_sub(1);
            if *connections == 0 {
                data.notification_connections.remove(&user_id);
            }
        }

        Ok(notification_connection_stats(&data, user_id))
    }

    pub fn notification_connection_stats(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        Ok(notification_connection_stats(&data, user_id))
    }

    pub fn total_online_connections(&self) -> Result<usize, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        Ok(data.notification_connections.values().sum())
    }

    pub fn pending_notification_pushes(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<NotificationPush>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        Ok(data
            .pending_notification_pushes
            .get(&user_id)
            .cloned()
            .unwrap_or_default())
    }

    pub fn ack_notification_push(
        &self,
        user_id: Uuid,
        push_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        let pushes = data
            .pending_notification_pushes
            .get_mut(&user_id)
            .ok_or(ForumError::NotFound)?;
        let before = pushes.len();
        pushes.retain(|push| push.push_id != push_id);
        if pushes.len() == before {
            return Err(ForumError::NotFound);
        }

        Ok(notification_connection_stats(&data, user_id))
    }

    #[cfg(feature = "ssr")]
    fn connect_notification_socket_trusted(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let mut data = self.write_data()?;
        *data.notification_connections.entry(user_id).or_default() += 1;
        Ok(notification_connection_stats(&data, user_id))
    }

    #[cfg(feature = "ssr")]
    fn disconnect_notification_socket_trusted(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let mut data = self.write_data()?;
        if let Some(connections) = data.notification_connections.get_mut(&user_id) {
            *connections = connections.saturating_sub(1);
            if *connections == 0 {
                data.notification_connections.remove(&user_id);
            }
        }
        Ok(notification_connection_stats(&data, user_id))
    }

    #[cfg(feature = "ssr")]
    fn notification_connection_stats_trusted(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        Ok(notification_connection_stats(&data, user_id))
    }

    #[cfg(feature = "ssr")]
    fn pending_notification_pushes_trusted(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<NotificationPush>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        Ok(data
            .pending_notification_pushes
            .get(&user_id)
            .cloned()
            .unwrap_or_default())
    }

    #[cfg(feature = "ssr")]
    fn ack_notification_push_trusted(
        &self,
        user_id: Uuid,
        push_id: Uuid,
    ) -> Result<NotificationConnectionStats, ForumError> {
        let mut data = self.write_data()?;
        let pushes = data
            .pending_notification_pushes
            .get_mut(&user_id)
            .ok_or(ForumError::NotFound)?;
        let before = pushes.len();
        pushes.retain(|push| push.push_id != push_id);
        if pushes.len() == before {
            return Err(ForumError::NotFound);
        }
        Ok(notification_connection_stats(&data, user_id))
    }

    fn toggle_post_set(
        &self,
        user_id: Uuid,
        post_id: Uuid,
        kind: ReactionKind,
    ) -> Result<ToggleResult, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        if !data.posts.contains_key(&post_id) {
            return Err(ForumError::NotFound);
        }

        let key = (user_id, post_id);
        let active = match kind {
            ReactionKind::Like => ReactionService::toggle_pair(&mut data.liked_posts, key),
            ReactionKind::Favorite => ReactionService::toggle_pair(&mut data.favorited_posts, key),
        };

        let post = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
        let post_author_id = post.summary.author_id;
        let post_title = post.summary.title.clone();
        let count = match kind {
            ReactionKind::Like => {
                ReactionService::apply_counter_delta(&mut post.summary.like_count, active)
            }
            ReactionKind::Favorite => {
                ReactionService::apply_counter_delta(&mut post.summary.favorite_count, active)
            }
        };

        if matches!(kind, ReactionKind::Like) && active && user_id != post_author_id {
            let actor_name = data
                .users
                .get(&user_id)
                .map(|user| user.nickname.clone())
                .unwrap_or_else(|| "有人".to_string());
            push_notification(
                &mut data,
                post_author_id,
                Some(user_id),
                NotificationType::PostLiked,
                format!("{actor_name} 喜欢了你的帖子"),
                post_title,
            );
        }

        Ok(ToggleResult { active, count })
    }

    fn set_post_status(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
        status: PostStatus,
    ) -> Result<ModerationPostAction, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        let was_pinned = data.pinned_posts.contains(&post_id);
        let was_recommended = data.recommended_posts.contains(&post_id);
        let post = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
        let action =
            ModerationService::apply_post_status(post, status.clone(), was_pinned, was_recommended);
        if status == PostStatus::Deleted {
            data.pinned_posts.remove(&post_id);
            data.recommended_posts.remove(&post_id);
        }
        Ok(action)
    }

    fn set_post_pin(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
        pinned: bool,
    ) -> Result<ModerationPostAction, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        let recommended = data.recommended_posts.contains(&post_id);
        let post = data
            .posts
            .get(&post_id)
            .ok_or(ForumError::NotFound)?
            .clone();
        let action = ModerationService::build_pin_action(&post, pinned, recommended)?;
        if pinned {
            data.pinned_posts.insert(post_id);
        } else {
            data.pinned_posts.remove(&post_id);
        }
        Ok(action)
    }

    fn set_post_recommend(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
        recommended: bool,
    ) -> Result<ModerationPostAction, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        let pinned = data.pinned_posts.contains(&post_id);
        let post = data
            .posts
            .get(&post_id)
            .ok_or(ForumError::NotFound)?
            .clone();
        let action = ModerationService::build_recommend_action(&post, recommended, pinned)?;
        if recommended {
            data.recommended_posts.insert(post_id);
        } else {
            data.recommended_posts.remove(&post_id);
        }
        Ok(action)
    }

    fn set_post_lock(
        &self,
        admin_id: Uuid,
        post_id: Uuid,
        locked: bool,
    ) -> Result<ModerationPostAction, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;
        let pinned = data.pinned_posts.contains(&post_id);
        let recommended = data.recommended_posts.contains(&post_id);
        let post = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
        let action = ModerationService::build_lock_action(post, locked, pinned, recommended)?;
        Ok(action)
    }

    fn set_comment_deleted(
        &self,
        admin_id: Uuid,
        comment_id: Uuid,
        deleted: bool,
    ) -> Result<ModerationCommentAction, ForumError> {
        let mut data = self.write_data()?;
        ensure_admin(&data, admin_id)?;

        for (post_id, comments) in data.comments.iter_mut() {
            if let Some(comment) = find_comment_mut(comments, comment_id) {
                let effect = ModerationService::apply_comment_deleted(comment, deleted);
                let post_id = *post_id;
                if effect.count_delta != 0 {
                    if let Some(post) = data.posts.get_mut(&post_id) {
                        ModerationService::apply_comment_count_delta(post, effect.count_delta);
                    }
                }
                return Ok(effect.action);
            }
        }
        Err(ForumError::NotFound)
    }

    fn write_data(&self) -> Result<std::sync::RwLockWriteGuard<'_, ForumData>, ForumError> {
        self.inner.write().map_err(|_| ForumError::Internal)
    }
}

#[derive(Clone, Copy)]
enum ReactionKind {
    Like,
    Favorite,
}

fn next_uuid(data: &mut ForumData) -> Uuid {
    let id = data.next_id;
    data.next_id += 1;
    Uuid::from_u128(id)
}

fn push_notification(
    data: &mut ForumData,
    recipient_id: Uuid,
    actor_id: Option<Uuid>,
    notification_type: NotificationType,
    title: String,
    body: String,
) {
    let notification = Notification {
        notification_id: next_uuid(data),
        recipient_id,
        actor_id,
        notification_type,
        title,
        body,
        read_at: None,
        created_at: OffsetDateTime::now_utc(),
    };
    data.notifications
        .entry(recipient_id)
        .or_default()
        .push(notification.clone());
    let online_connections = data
        .notification_connections
        .get(&recipient_id)
        .copied()
        .unwrap_or_default();
    if online_connections > 0 {
        if let Some(push) = NotificationPushService::build_pending_push(
            next_uuid(data),
            online_connections,
            notification,
        ) {
            data.pending_notification_pushes
                .entry(recipient_id)
                .or_default()
                .push(push);
        }
    }
}

fn notification_connection_stats(data: &ForumData, user_id: Uuid) -> NotificationConnectionStats {
    NotificationConnectionStats {
        user_id,
        online_connections: data
            .notification_connections
            .get(&user_id)
            .copied()
            .unwrap_or_default(),
        pending_push_count: data
            .pending_notification_pushes
            .get(&user_id)
            .map(Vec::len)
            .unwrap_or_default(),
    }
}

fn seed_category_items() -> HashMap<Uuid, CategoryItem> {
    [
        category_seed(1001, "公告", "#0064E0", 1, 12),
        category_seed(1002, "教程", "#35A853", 2, 34),
        category_seed(1003, "问题", "#F97316", 3, 156),
        category_seed(1004, "经验分享", "#7CB4FF", 4, 78),
        category_seed(1005, "讨论", "#9CA3AF", 5, 45),
        category_seed(1006, "站务", "#A855F7", 6, 17),
    ]
    .into_iter()
    .map(|category| (category.category_id, category))
    .collect()
}

fn category_seed(
    id: u128,
    name: &str,
    color: &str,
    sort_order: i32,
    post_count: u32,
) -> CategoryItem {
    CategoryItem {
        category_id: Uuid::from_u128(id),
        name: name.to_string(),
        color: color.to_string(),
        sort_order,
        enabled: true,
        post_count,
    }
}

fn seed_roles() -> HashMap<String, Role> {
    let mut roles = HashMap::new();
    roles.insert(
        "admin".to_string(),
        Role {
            code: "admin".to_string(),
            name: "管理员".to_string(),
            permissions: admin_permissions(),
        },
    );
    roles.insert(
        "member".to_string(),
        Role {
            code: "member".to_string(),
            name: "普通用户".to_string(),
            permissions: RbacService::resolve_permissions(&[
                "post:view".to_string(),
                "comment:view".to_string(),
            ])
            .expect("seed member permissions"),
        },
    );
    roles.insert(
        "moderator".to_string(),
        Role {
            code: "moderator".to_string(),
            name: "内容审核员".to_string(),
            permissions: RbacService::resolve_permissions(&[
                "post:view".to_string(),
                "post:update".to_string(),
                "comment:view".to_string(),
                "comment:delete".to_string(),
                "report:view".to_string(),
            ])
            .expect("seed moderator permissions"),
        },
    );
    roles.insert(
        "operator".to_string(),
        Role {
            code: "operator".to_string(),
            name: "运营人员".to_string(),
            permissions: RbacService::resolve_permissions(&[
                "announcement:create".to_string(),
                "announcement:publish".to_string(),
                "category:view".to_string(),
                "tag:view".to_string(),
            ])
            .expect("seed operator permissions"),
        },
    );
    roles
}

fn sorted_roles(roles: impl IntoIterator<Item = Role>) -> Vec<Role> {
    let mut roles = roles.into_iter().collect::<Vec<_>>();
    roles.sort_by(|left, right| left.code.cmp(&right.code));
    roles
}

fn role_snapshot(role: &Role) -> String {
    format!(
        "code={},name={},permissions={}",
        role.code,
        role.name,
        role.permissions
            .iter()
            .map(|permission| permission.code.as_str())
            .collect::<Vec<_>>()
            .join("|")
    )
}

fn seed_tag_items() -> HashMap<Uuid, TagItem> {
    [
        tag_seed(2001, "leptos", 1, 132),
        tag_seed(2002, "axum", 2, 98),
        tag_seed(2003, "sqlx", 3, 86),
        tag_seed(2004, "postgresql", 4, 64),
        tag_seed(2005, "rust", 5, 61),
        tag_seed(2006, "wasm", 6, 48),
        tag_seed(2007, "server-functions", 7, 42),
        tag_seed(2008, "markdown", 8, 38),
    ]
    .into_iter()
    .map(|tag| (tag.tag_id, tag))
    .collect()
}

fn tag_seed(id: u128, name: &str, sort_order: i32, use_count: u32) -> TagItem {
    TagItem {
        tag_id: Uuid::from_u128(id),
        name: name.to_string(),
        sort_order,
        enabled: true,
        use_count,
    }
}

fn admin_user_rows(data: &ForumData) -> Vec<AdminUserRow> {
    let mut rows = data
        .users
        .keys()
        .filter_map(|user_id| admin_user_row(data, *user_id))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.username.cmp(&right.username));
    rows
}

fn admin_user_row(data: &ForumData, user_id: Uuid) -> Option<AdminUserRow> {
    let user = data.users.get(&user_id)?;
    let roles = data
        .user_roles
        .get(&user_id)
        .cloned()
        .unwrap_or_else(|| vec!["member".to_string()]);
    let post_count = data
        .posts
        .values()
        .filter(|post| post.summary.author_id == user_id && post.status != PostStatus::Deleted)
        .count();
    let comment_count = data
        .comments
        .values()
        .map(|comments| count_user_comments(comments, user_id))
        .sum();

    Some(UserAdminService::admin_user_row(
        user,
        roles,
        data.disabled_users.contains(&user_id),
        post_count,
        comment_count,
    ))
}

fn user_audit_snapshot(data: &ForumData, user_id: Uuid) -> Option<String> {
    admin_user_row(data, user_id).map(|row| UserAdminService::audit_snapshot(&row))
}

fn push_audit_log(
    data: &mut ForumData,
    actor: &SessionUser,
    action: &str,
    target_type: &str,
    target_id: Uuid,
    target_label: String,
    before: Option<String>,
    after: Option<String>,
    context: AuditContext,
) {
    let audit_id = next_uuid(data);
    data.audit_logs.push(UserAdminService::build_audit_log(
        audit_id,
        actor,
        action,
        target_type,
        target_id,
        target_label,
        before,
        after,
        context,
        OffsetDateTime::now_utc(),
    ));
}

fn count_user_comments(comments: &[CommentNode], user_id: Uuid) -> usize {
    comments
        .iter()
        .map(|comment| {
            usize::from(comment.author_id == user_id && !comment.deleted)
                + count_user_comments(&comment.replies, user_id)
        })
        .sum()
}

fn find_comment_mut(comments: &mut [CommentNode], comment_id: Uuid) -> Option<&mut CommentNode> {
    for comment in comments {
        if comment.comment_id == comment_id {
            return Some(comment);
        }
        if let Some(found) = find_comment_mut(&mut comment.replies, comment_id) {
            return Some(found);
        }
    }
    None
}

fn sorted_categories(categories: impl IntoIterator<Item = CategoryItem>) -> Vec<CategoryItem> {
    let mut categories = categories.into_iter().collect::<Vec<_>>();
    categories.sort_by(|left, right| {
        left.sort_order
            .cmp(&right.sort_order)
            .then_with(|| left.name.cmp(&right.name))
    });
    categories
}

fn sorted_tags(tags: impl IntoIterator<Item = TagItem>) -> Vec<TagItem> {
    let mut tags = tags.into_iter().collect::<Vec<_>>();
    tags.sort_by(|left, right| {
        right
            .use_count
            .cmp(&left.use_count)
            .then_with(|| left.sort_order.cmp(&right.sort_order))
            .then_with(|| left.name.cmp(&right.name))
    });
    tags
}

fn ensure_category_name_unique(
    data: &ForumData,
    name: &str,
    ignore_id: Option<Uuid>,
) -> Result<(), ForumError> {
    let normalized = name.trim();
    let exists = data.categories.values().any(|category| {
        Some(category.category_id) != ignore_id && category.enabled && category.name == normalized
    });
    if exists {
        return Err(ForumError::Conflict("分类名称已存在".to_string()));
    }
    Ok(())
}

fn ensure_tag_name_unique(
    data: &ForumData,
    name: &str,
    ignore_id: Option<Uuid>,
) -> Result<(), ForumError> {
    let normalized = name.trim().to_lowercase();
    let exists = data
        .tags
        .values()
        .any(|tag| Some(tag.tag_id) != ignore_id && tag.enabled && tag.name == normalized);
    if exists {
        return Err(ForumError::Conflict("标签名称已存在".to_string()));
    }
    Ok(())
}

fn announcement_recipients(data: &ForumData, announcement: &AnnouncementItem) -> Vec<Uuid> {
    match &announcement.audience {
        AnnouncementAudience::AllUsers => data.users.keys().copied().collect(),
        AnnouncementAudience::UserIds(user_ids) => user_ids
            .iter()
            .copied()
            .filter(|user_id| data.users.contains_key(user_id))
            .collect(),
    }
}

fn sorted_announcements(
    announcements: impl IntoIterator<Item = AnnouncementItem>,
) -> Vec<AnnouncementItem> {
    let mut announcements = announcements.into_iter().collect::<Vec<_>>();
    announcements.sort_by(|left, right| {
        right
            .pinned
            .cmp(&left.pinned)
            .then_with(|| right.updated_at.cmp(&left.updated_at))
    });
    announcements
}

fn home_announcement(announcement: AnnouncementItem) -> HomeAnnouncement {
    HomeAnnouncement {
        title: announcement.title,
        date_label: announcement
            .published_at
            .unwrap_or(announcement.updated_at)
            .date()
            .to_string(),
    }
}

fn ensure_admin(data: &ForumData, user_id: Uuid) -> Result<&SessionUser, ForumError> {
    let user = data.users.get(&user_id).ok_or(ForumError::Unauthorized)?;
    if !user.is_admin {
        return Err(ForumError::Forbidden);
    }
    Ok(user)
}

fn report_target_title(
    data: &ForumData,
    request: &CreateReportRequest,
) -> Result<Option<String>, ForumError> {
    match request.target_type {
        ReportTargetType::Post => data
            .posts
            .get(&request.target_id)
            .map(|post| Some(post.summary.title.clone()))
            .ok_or(ForumError::NotFound),
        ReportTargetType::Comment => data
            .comments
            .values()
            .find_map(|comments| find_comment(comments, request.target_id))
            .map(|comment| Some(comment.content.chars().take(40).collect()))
            .ok_or(ForumError::NotFound),
        ReportTargetType::User => data
            .users
            .get(&request.target_id)
            .map(|user| Some(user.nickname.clone()))
            .ok_or(ForumError::NotFound),
    }
}

fn find_comment(comments: &[CommentNode], comment_id: Uuid) -> Option<&CommentNode> {
    for comment in comments {
        if comment.comment_id == comment_id {
            return Some(comment);
        }
        if let Some(found) = find_comment(&comment.replies, comment_id) {
            return Some(found);
        }
    }
    None
}

fn find_comment_with_post(
    comments_by_post: &HashMap<Uuid, Vec<CommentNode>>,
    comment_id: Uuid,
) -> Option<(Uuid, &CommentNode)> {
    comments_by_post.iter().find_map(|(post_id, comments)| {
        find_comment(comments, comment_id).map(|comment| (*post_id, comment))
    })
}

fn user_profile(data: &ForumData, user: &SessionUser) -> UserProfile {
    UserProfile {
        user_id: user.user_id,
        username: user.username.clone(),
        nickname: user.nickname.clone(),
        avatar_url: user.avatar_url.clone(),
        bio: data
            .user_bios
            .get(&user.user_id)
            .cloned()
            .unwrap_or_else(|| "热爱 Rust 全栈开发的社区成员。".to_string()),
        registered_at: data
            .user_registered_at
            .get(&user.user_id)
            .copied()
            .unwrap_or_else(|| OffsetDateTime::now_utc() - Duration::days(30)),
    }
}

fn update_comment_author_name(comments: &mut [CommentNode], user_id: Uuid, nickname: &str) {
    for comment in comments {
        if comment.author_id == user_id {
            comment.author_name = nickname.to_string();
        }
        update_comment_author_name(&mut comment.replies, user_id, nickname);
    }
}

fn flatten_user_comments(
    post_id: Uuid,
    post_title: &str,
    comments: &[CommentNode],
    user_id: Uuid,
) -> Vec<UserCommentItem> {
    let mut items = Vec::new();
    for comment in comments {
        if comment.author_id == user_id {
            items.push(UserCommentItem {
                post_id,
                post_title: post_title.to_string(),
                content: comment.content.clone(),
                created_at: comment.created_at,
            });
        }
        items.extend(flatten_user_comments(
            post_id,
            post_title,
            &comment.replies,
            user_id,
        ));
    }
    items
}

fn shared_tag_count(tags: &[String], source_tags: &HashSet<String>) -> usize {
    tags.iter()
        .filter(|tag| source_tags.contains(&tag.to_lowercase()))
        .count()
}

fn render_markdown_safe(markdown: &str) -> String {
    markdown
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }

            let escaped = escape_html(line);
            if let Some(heading) = escaped.strip_prefix("# ") {
                Some(format!("<h1>{heading}</h1>"))
            } else if let Some(heading) = escaped.strip_prefix("## ") {
                Some(format!("<h2>{heading}</h2>"))
            } else if let Some(quote) = escaped.strip_prefix("&gt; ") {
                Some(format!("<blockquote>{quote}</blockquote>"))
            } else {
                Some(format!("<p>{escaped}</p>"))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn append_reply(
    comments: &mut [CommentNode],
    parent_id: Uuid,
    reply: CommentNode,
) -> Result<(), ForumError> {
    for comment in comments {
        if comment.comment_id == parent_id {
            comment.replies.push(reply);
            return Ok(());
        }
        if append_reply(&mut comment.replies, parent_id, reply.clone()).is_ok() {
            return Ok(());
        }
    }
    Err(ForumError::NotFound)
}
