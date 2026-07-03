use jiff::Timestamp;
use toasty::Db;
use uuid::Uuid;

#[derive(Debug, Clone, toasty::Model)]
#[table = "conversation"]
pub struct Conversation {
    #[key]
    #[auto]
    pub id: Uuid,
    pub doc_id: String,
    pub doc_type: String,
    pub user_id: i64,
    pub title: String,
    #[column("type")]
    pub r#type: String,
    pub inline_type: Option<String>,
    #[column(type = timestamp(6))]
    pub created_at: Timestamp,
    #[column(type = timestamp(6))]
    pub updated_at: Timestamp,
    pub deleted_at: i64,
}

#[derive(Debug, Clone, toasty::Model)]
#[table = "turn"]
pub struct Turn {
    #[key]
    #[auto]
    pub id: Uuid,
    pub conversation_id: Uuid,
    #[column(type = text)]
    pub input_context: String,
    pub document_content_version_id: i64,
    #[column(type = timestamp(6))]
    pub created_at: Timestamp,
    #[column(type = timestamp(6))]
    pub updated_at: Timestamp,
    pub deleted_at: i64,
}

#[derive(Debug, Clone, toasty::Model)]
#[table = "turn_response"]
pub struct TurnResponse {
    #[key]
    #[auto]
    pub id: Uuid,
    pub turn_id: Uuid,
    #[column("type")]
    pub r#type: String,
    #[column(type = text)]
    pub response: String,
    #[column(type = timestamp(6))]
    pub created_at: Timestamp,
}

#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ConversationPageParam {
    pub user_id: Option<i64>,
    pub doc_id: Option<String>,
    pub doc_type: Option<String>,
    pub r#type: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn connect_database(url: &str) -> toasty::Result<Db> {
    toasty::Db::builder()
        .models(toasty::models!(Conversation, Turn, TurnResponse))
        .connect(url)
        .await
}

pub async fn create_conversation(
    db: &mut Db,
    conversation: Conversation,
) -> toasty::Result<Conversation> {
    Conversation::create()
        .doc_id(conversation.doc_id)
        .doc_type(conversation.doc_type)
        .user_id(conversation.user_id)
        .title(conversation.title)
        .r#type(conversation.r#type)
        .inline_type(conversation.inline_type)
        .created_at(conversation.created_at)
        .updated_at(conversation.updated_at)
        .deleted_at(conversation.deleted_at)
        .exec(db)
        .await
}

pub async fn get_conversation_by_id(db: &mut Db, id: Uuid) -> toasty::Result<Conversation> {
    Conversation::filter_by_id(id).get(db).await
}

pub async fn page_conversations(
    db: &mut Db,
    param: ConversationPageParam,
) -> toasty::Result<Page<Conversation>> {
    let page = param.page.unwrap_or(1).max(1);
    let page_size = param.page_size.unwrap_or(20).max(1);
    let limit = usize::try_from(page_size).unwrap_or(usize::MAX);
    let offset = usize::try_from((page - 1) * page_size).unwrap_or(usize::MAX);

    let query = conversation_page_query(param);
    let total = query.clone().count().exec(db).await?;

    let items = query
        .order_by((
            Conversation::fields().updated_at().desc(),
            Conversation::fields().id().desc(),
        ))
        .limit(limit)
        .offset(offset)
        .exec(db)
        .await?;

    Ok(Page {
        items,
        total: i64::try_from(total).unwrap_or(i64::MAX),
        page,
        page_size,
    })
}

pub async fn delete_conversation(db: &mut Db, id: Uuid) -> toasty::Result<()> {
    let timestamp = Timestamp::now().as_millisecond();

    Conversation::filter_by_id(id)
        .filter(Conversation::fields().deleted_at().eq(0))
        .update()
        .deleted_at(timestamp)
        .exec(db)
        .await?;

    Ok(())
}

pub async fn create_turn(db: &mut Db, turn: Turn) -> toasty::Result<Turn> {
    Turn::create()
        .conversation_id(turn.conversation_id)
        .input_context(turn.input_context)
        .document_content_version_id(turn.document_content_version_id)
        .created_at(turn.created_at)
        .updated_at(turn.updated_at)
        .deleted_at(turn.deleted_at)
        .exec(db)
        .await
}

pub async fn select_turn_by_conversation_for_page(
    db: &mut Db,
    conversation_id: Uuid,
    page_num: i64,
    page_size: i64,
) -> toasty::Result<Page<Turn>> {
    let page = page_num.max(1);
    let page_size = page_size.max(1);
    let limit = usize::try_from(page_size).unwrap_or(usize::MAX);
    let offset = usize::try_from((page - 1) * page_size).unwrap_or(usize::MAX);
    let query = Turn::filter(Turn::fields().conversation_id().eq(conversation_id));
    let total = query.clone().count().exec(db).await?;

    let items = query.limit(limit).offset(offset).exec(db).await?;

    Ok(Page {
        items,
        total: i64::try_from(total).unwrap_or(i64::MAX),
        page,
        page_size,
    })
}

pub async fn batch_insert_turn_response(
    db: &mut Db,
    response_list: Vec<TurnResponse>,
) -> toasty::Result<()> {
    if response_list.is_empty() {
        return Ok(());
    }

    let mut create_many = TurnResponse::create_many();

    for response in response_list {
        create_many = create_many.with_item(|item| {
            item.turn_id(response.turn_id)
                .r#type(response.r#type)
                .response(response.response)
                .created_at(response.created_at)
        });
    }

    create_many.exec(db).await?;

    Ok(())
}

pub async fn get_turn_response_list_by_turn_id(
    db: &mut Db,
    turn_id: Uuid,
) -> toasty::Result<Vec<TurnResponse>> {
    TurnResponse::filter(TurnResponse::fields().turn_id().eq(turn_id))
        .order_by(TurnResponse::fields().created_at().asc())
        .exec(db)
        .await
}

fn conversation_page_query(
    param: ConversationPageParam,
) -> <Conversation as toasty::schema::Model>::Query {
    let mut query = Conversation::all().filter(Conversation::fields().deleted_at().eq(0));

    if let Some(user_id) = param.user_id {
        query = query.filter(Conversation::fields().user_id().eq(user_id));
    }
    if let Some(doc_id) = param.doc_id {
        query = query.filter(Conversation::fields().doc_id().eq(doc_id));
    }
    if let Some(doc_type) = param.doc_type {
        query = query.filter(Conversation::fields().doc_type().eq(doc_type));
    }
    if let Some(conversation_type) = param.r#type {
        query = query.filter(Conversation::fields().r#type().eq(conversation_type));
    }

    query
}
