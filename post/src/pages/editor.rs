use leptos::prelude::*;

use crate::components::PageShell;

#[component]
pub fn EditorPage() -> impl IntoView {
    view! {
        <PageShell>
            <div class="editor-layout">
                <section class="editor-main">
                    <div class="page-kicker">"Markdown 编辑 / 帖子编辑"</div>
                    <div class="editor-title-row">
                        <input class="title-input" placeholder="输入帖子标题"/>
                        <span class="autosave-state">"自动保存 · 12 秒前"</span>
                    </div>
                    <textarea class="input summary-input" placeholder="摘要，可自动生成，也可手动填写"></textarea>
                    <div class="editor-meta-grid">
                        <select class="select select-bordered"><option>"选择分类"</option><option>"教程"</option><option>"问题"</option><option>"经验分享"</option></select>
                        <input class="input input-bordered" placeholder="标签，用逗号分隔"/>
                        <label class="upload-control">
                            <span>"上传封面图 · PNG/JPEG/WebP · ≤ 5MB"</span>
                            <input type="file" accept="image/png,image/jpeg,image/webp"/>
                        </label>
                    </div>
                    <div class="editor-toolbar" aria-label="编辑器工具栏">
                        <button>"撤销"</button>
                        <button>"重做"</button>
                        <button>"标题"</button>
                        <button>"引用"</button>
                        <button>"代码块"</button>
                        <button>"表格"</button>
                        <button>"有序列表"</button>
                        <button>"无序列表"</button>
                        <button>"插入图片"</button>
                    </div>
                    <textarea
                        class="textarea textarea-bordered markdown-input"
                        placeholder="使用 Markdown 编写正文，支持代码块、表格、引用、有序列表、无序列表和图片链接。"
                    ></textarea>
                    <div class="editor-actions">
                        <button class="btn btn-ghost">"自动保存草稿"</button>
                        <button class="btn btn-outline">"预览安全过滤"</button>
                        <button class="btn btn-outline">"删除自己的帖子"</button>
                        <button class="btn btn-primary">"发布帖子"</button>
                    </div>
                </section>
                <aside class="preview-panel">
                    <div class="panel-eyebrow">"实时预览"</div>
                    <h1>"从 PRD 到 Leptos 论坛"</h1>
                    <p>"Markdown 渲染后的 HTML 会做 XSS 过滤，图片上传会限制 MIME 类型和文件大小。"</p>
                    <div class="code-preview">
                        <span>"代码高亮"</span>
                        <pre><code>"fn main() {\n    println!(\"hello forum\");\n}"</code></pre>
                    </div>
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
