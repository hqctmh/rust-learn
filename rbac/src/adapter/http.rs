use axum::Json;

use crate::{di::AppState, domain::param::UserPageQuery};

async fn user_page_list(State(state):State<AppState>,Query(query):Query<UserPageQuery>)->Result<Json<Page>
