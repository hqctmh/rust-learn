use anyhow::Ok;
use chrono::Utc;
use rbac::di::AppState;

#[tokio::test]
async fn service_create_user() -> anyhow::Result<()> {
    let state = AppState::from_env().await?;

    let username = format!("test_user_{}", Utc::now().timestamp());

    let user = state
        .user_service
        .user_regist(username.as_str(), "111111", Some("test"))
        .await?;

    println!("user:{:?}", user);

    let user = state
        .user_repository
        .find_by_username(username.as_str())
        .await?;

    println!("find user:{:?}", user);
    Ok(())
}

#[tokio::test]
async fn service_user_login() -> anyhow::Result<()> {
    let state = AppState::from_env().await?;

    let username = "test_user_1785230715";

    let user = state
        .user_service
        .user_login(username, "111111", "123")
        .await?;
    println!("user:{:?}", user);

    Ok(())
}
