use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::components::PageShell;
use crate::domain::files::FileAsset;
use crate::domain::posts::PostStatus;
use crate::page_data::{
    SubmitPost, delete_editor_post, load_editor_post, preview_markdown, upload_editor_image,
};

#[derive(Clone, Debug)]
struct EditorImageUploadInput {
    session_id: String,
    original_filename: String,
    mime_type: String,
    content_base64: String,
}

#[derive(Clone, Debug)]
struct EditorDeletePostInput {
    session_id: String,
    post_id: String,
}

#[component]
pub fn EditorPage() -> impl IntoView {
    let query = use_query_map();
    let params = use_params_map();
    let submit_action = ServerAction::<SubmitPost>::new();
    let action = submit_action;
    let submit_pending = action.pending();
    let submit_result = action.value();
    let title_value = RwSignal::new(String::new());
    let summary_value = RwSignal::new(String::new());
    let category_value = RwSignal::new(String::new());
    let tag_names_value = RwSignal::new(String::new());
    let markdown_value = RwSignal::new(String::new());
    let preview_action = Action::new(|markdown: &String| preview_markdown(markdown.clone()));
    let preview_pending = preview_action.pending();
    let preview_result = preview_action.value();
    let upload_image_action = Action::new(|input: &EditorImageUploadInput| {
        let input = input.clone();
        upload_editor_image(
            input.session_id,
            input.original_filename,
            input.mime_type,
            input.content_base64,
        )
    });
    let upload_image_pending = upload_image_action.pending();
    let upload_image_result = upload_image_action.value();
    let delete_post_action = Action::new(|input: &EditorDeletePostInput| {
        let input = input.clone();
        delete_editor_post(input.session_id, input.post_id)
    });
    let delete_post_pending = delete_post_action.pending();
    let delete_post_result = delete_post_action.value();
    let session_id = move || query.read().get("session_id").unwrap_or_default();
    let post_id = move || params.read().get("id").unwrap_or_default();
    let editor_post = Resource::new(
        move || (session_id(), post_id()),
        |(session_id, post_id)| load_editor_post(session_id, post_id),
    );
    let editor_loaded = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(Ok(asset)) = upload_image_result.get() {
            append_markdown_image(markdown_value, &asset);
        }
    });

    Effect::new(move |_| {
        if editor_loaded.get_untracked() {
            return;
        }
        if let Some(Ok(Some(post))) = editor_post.get() {
            title_value.set(post.summary.title.clone());
            summary_value.set(post.summary.summary.clone());
            category_value.set(post.summary.category_name.clone().unwrap_or_default());
            tag_names_value.set(post.summary.tags.join(", "));
            markdown_value.set(post.markdown.clone());
            editor_loaded.set(true);
        }
    });

    view! {
        <PageShell>
            <div class="editor-layout">
                <section class="editor-main">
                    <div class="page-kicker">"Markdown 编辑 / 帖子编辑"</div>
                    <ActionForm action=submit_action>
                        <div class="editor-session-row">
                            <label>
                                <span>"会话 ID"</span>
                                <input class="input input-bordered" name="session_id" value=session_id placeholder="登录或注册后自动带入，也可手动粘贴"/>
                            </label>
                        </div>
                        <input type="hidden" name="post_id" value=post_id/>
                        <div class="editor-title-row">
                            <input
                                class="title-input"
                                name="title"
                                placeholder="输入帖子标题"
                                prop:value=move || title_value.get()
                                on:input=move |event| title_value.set(event_target_value(&event))
                            />
                            <span class="autosave-state">"自动保存 · 12 秒前"</span>
                        </div>
                        <textarea
                            class="input summary-input"
                            name="summary"
                            placeholder="摘要，可自动生成，也可手动填写"
                            prop:value=move || summary_value.get()
                            on:input=move |event| summary_value.set(event_target_value(&event))
                        ></textarea>
                        <div class="editor-meta-grid">
                            <select
                                class="select select-bordered"
                                name="category_name"
                                prop:value=move || category_value.get()
                                on:change=move |event| category_value.set(event_target_value(&event))
                            >
                                <option value="">"选择分类"</option>
                                <option value="教程">"教程"</option>
                                <option value="问题">"问题"</option>
                                <option value="经验分享">"经验分享"</option>
                                <option value="讨论">"讨论"</option>
                            </select>
                            <input
                                class="input input-bordered"
                                name="tag_names"
                                placeholder="标签，用逗号分隔"
                                prop:value=move || tag_names_value.get()
                                on:input=move |event| tag_names_value.set(event_target_value(&event))
                            />
                            <label class="upload-control">
                                <span>"上传封面图 · PNG/JPEG/WebP · ≤ 5MB"</span>
                                <input type="file" accept="image/png,image/jpeg,image/webp"/>
                            </label>
                        </div>
                        <div class="editor-toolbar" aria-label="编辑器工具栏">
                            <button type="button">"撤销"</button>
                            <button type="button">"重做"</button>
                            <button type="button">"标题"</button>
                            <button type="button">"引用"</button>
                            <button type="button">"代码块"</button>
                            <button type="button">"表格"</button>
                            <button type="button">"有序列表"</button>
                            <button type="button">"无序列表"</button>
                            <label class="editor-upload-button">
                                <span>{move || if upload_image_pending.get() { "上传中..." } else { "插入图片" }}</span>
                                <input
                                    type="file"
                                    accept="image/png,image/jpeg,image/webp"
                                    disabled=move || upload_image_pending.get()
                                    on:change=move |event| {
                                        let session_id = session_id();
                                        queue_image_upload_from_event(event, session_id, move |input| {
                                            upload_image_action.dispatch(input);
                                        });
                                    }
                                />
                            </label>
                        </div>
                        <textarea
                            class="textarea textarea-bordered markdown-input"
                            name="markdown"
                            placeholder="使用 Markdown 编写正文，支持代码块、表格、引用、有序列表、无序列表和图片链接。"
                            prop:value=move || markdown_value.get()
                            on:input=move |event| markdown_value.set(event_target_value(&event))
                        ></textarea>
                        <div class="editor-actions">
                            <button class="btn btn-ghost" type="submit" name="save_mode" value="draft" disabled=move || submit_pending.get()>
                                {move || if submit_pending.get() { "保存中..." } else { "自动保存草稿" }}
                            </button>
                            <button
                                class="btn btn-outline"
                                type="button"
                                disabled=move || preview_pending.get()
                                on:click=move |_| {
                                    preview_action.dispatch(markdown_value.get());
                                }
                            >
                                {move || if preview_pending.get() { "预览中..." } else { "预览安全过滤" }}
                            </button>
                            <button
                                class="btn btn-outline"
                                type="button"
                                disabled=move || delete_post_pending.get() || post_id().is_empty()
                                on:click=move |_| {
                                    delete_post_action.dispatch(EditorDeletePostInput {
                                        session_id: session_id(),
                                        post_id: post_id(),
                                    });
                                }
                            >
                                {move || if delete_post_pending.get() { "删除中..." } else { "删除自己的帖子" }}
                            </button>
                            <button class="btn btn-primary" type="submit" name="save_mode" value="publish" disabled=move || submit_pending.get()>
                                {move || if submit_pending.get() { "发布中..." } else { "发布帖子" }}
                            </button>
                        </div>
                    </ActionForm>
                    {move || {
                        submit_result.get().map(|result| match result {
                            Ok(post) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>{if post.status == PostStatus::Draft { "草稿保存成功" } else { "发布成功" }}</strong>
                                    <span>{post.summary.title}</span>
                                    <a href=format!("/posts/{}", post.summary.post_id)>"查看帖子"</a>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"发布失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                    {move || {
                        upload_image_result.get().map(|result| match result {
                            Ok(asset) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>"图片上传成功"</strong>
                                    <span>{asset.markdown_image}</span>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"图片上传失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                    {move || {
                        delete_post_result.get().map(|result| match result {
                            Ok(post) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>"删除成功"</strong>
                                    <span>{post.summary.title}</span>
                                    <a href="/">"返回首页"</a>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"删除失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                </section>
                <aside class="preview-panel">
                    <div class="panel-eyebrow">"实时预览"</div>
                    {move || {
                        preview_result.get().map(|result| match result {
                            Ok(html) => view! {
                                <div class="preview-rendered article-body" inner_html=html></div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"预览失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        }).unwrap_or_else(|| view! {
                            <div class="preview-placeholder">
                                <h1>"从 PRD 到 Leptos 论坛"</h1>
                                <p>"Markdown 渲染后的 HTML 会做 XSS 过滤，图片上传会限制 MIME 类型和文件大小。"</p>
                                <div class="code-preview">
                                    <span>"代码高亮"</span>
                                    <pre><code>"fn main() {\n    println!(\"hello forum\");\n}"</code></pre>
                                </div>
                            </div>
                        }.into_any())
                    }}
                    <div class="preview-checklist">
                        <span>"RustFS 图片存储"</span>
                        <span>"MIME 类型校验"</span>
                        <span>"文件大小限制"</span>
                        <span>"生成 Markdown 图片链接"</span>
                        <span>"草稿自动保存"</span>
                        <span>"作者编辑权限校验"</span>
                        <span>"作者删除自己的帖子"</span>
                        <span>"发布后同步 Elasticsearch"</span>
                        <span>"NATS 通知作者粉丝"</span>
                    </div>
                    <div class="upload-contract">
                        <span>"上传返回 URL"</span>
                        <code>"![cover.png](/uploads/markdown/sha256demo/cover.png)"</code>
                    </div>
                </aside>
            </div>
        </PageShell>
    }
}

fn append_markdown_image(markdown_value: RwSignal<String>, asset: &FileAsset) {
    markdown_value.update(|markdown| {
        if !markdown.is_empty() && !markdown.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push_str(&asset.markdown_image);
        markdown.push('\n');
    });
}

#[cfg(feature = "hydrate")]
fn queue_image_upload_from_event(
    event: leptos::ev::Event,
    session_id: String,
    dispatch: impl Fn(EditorImageUploadInput) + Clone + 'static,
) {
    use wasm_bindgen::{JsCast, closure::Closure};

    let Some(input) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return;
    };
    let Some(file) = input
        .files()
        .and_then(|files: web_sys::FileList| files.item(0))
    else {
        return;
    };
    let Ok(reader) = web_sys::FileReader::new() else {
        return;
    };

    let reader_for_callback = reader.clone();
    let original_filename = file.name();
    let mime_type = file.type_();
    let dispatch_upload = dispatch.clone();
    let onloadend = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        let Some(data_url) = reader_for_callback
            .result()
            .ok()
            .and_then(|value| value.as_string())
        else {
            return;
        };
        let content_base64 = data_url
            .split_once(',')
            .map(|(_, content)| content.to_string())
            .unwrap_or(data_url);
        dispatch_upload(EditorImageUploadInput {
            session_id: session_id.clone(),
            original_filename: original_filename.clone(),
            mime_type: mime_type.clone(),
            content_base64,
        });
    }));

    reader.set_onloadend(Some(onloadend.as_ref().unchecked_ref()));
    if reader.read_as_data_url(&file).is_ok() {
        onloadend.forget();
    }
}

#[cfg(not(feature = "hydrate"))]
fn queue_image_upload_from_event(
    _event: leptos::ev::Event,
    _session_id: String,
    _dispatch: impl Fn(EditorImageUploadInput) + Clone + 'static,
) {
}
