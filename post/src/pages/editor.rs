use leptos::prelude::*;

use crate::components::PageShell;

#[component]
pub fn EditorPage() -> impl IntoView {
    view! {
        <PageShell>
            <div class="grid gap-6 lg:grid-cols-[1fr_420px]">
                <section class="space-y-4">
                    <input class="input input-bordered w-full text-lg font-semibold" placeholder="输入帖子标题"/>
                    <div class="grid gap-4 sm:grid-cols-2">
                        <select class="select select-bordered"><option>"选择分类"</option><option>"Leptos"</option><option>"实践"</option></select>
                        <input class="input input-bordered" placeholder="标签，用逗号分隔"/>
                    </div>
                    <textarea class="textarea textarea-bordered min-h-[420px] w-full font-mono" placeholder="使用 Markdown 编写正文，支持代码块、表格、引用和图片链接。"></textarea>
                    <div class="flex justify-end gap-3">
                        <button class="btn btn-ghost">"保存草稿"</button>
                        <button class="btn btn-primary">"发布帖子"</button>
                    </div>
                </section>
                <aside class="rounded-lg border border-base-300 p-5">
                    <h1 class="mb-3 text-lg font-semibold">"实时预览"</h1>
                    <p class="text-sm leading-6 text-base-content/70">"Markdown 预览会在这里显示。服务端渲染前会清洗 HTML，避免 XSS。"</p>
                    <pre class="mt-5 rounded-md bg-base-200 p-4 text-sm"><code>"```rust\nprintln!(\"hello forum\");\n```"</code></pre>
                </aside>
            </div>
        </PageShell>
    }
}
