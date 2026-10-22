use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Table(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    // `min-w`: on narrow screens the table scrolls horizontally inside its wrapper instead of crushing columns.
    let base = "w-full min-w-[48rem] caption-bottom text-sm";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class="relative w-full overflow-x-auto">
            <table class=merged_class>
                {children()}
            </table>
        </div>
    }
}

#[component]
pub fn TableHeader(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "[&_tr]:border-b border-border bg-muted/50";
    let merged_class = tw_merge!(base, &class);

    view! {
        <thead class=merged_class>
            {children()}
        </thead>
    }
}

#[component]
pub fn TableBody(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "[&_tr:last-child]:border-0";
    let merged_class = tw_merge!(base, &class);

    view! {
        <tbody class=merged_class>
            {children()}
        </tbody>
    }
}

#[component]
pub fn TableFooter(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "border-t border-border bg-muted/50 font-medium [&>tr]:last:border-b-0";
    let merged_class = tw_merge!(base, &class);

    view! {
        <tfoot class=merged_class>
            {children()}
        </tfoot>
    }
}

#[component]
pub fn TableRow(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base =
        "border-b border-border transition-colors hover:bg-accent/60 data-[state=selected]:bg-muted";
    let merged_class = tw_merge!(base, &class);

    view! {
        <tr class=merged_class>
            {children()}
        </tr>
    }
}

#[component]
pub fn TableHead(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "h-10 px-4 text-left align-middle whitespace-nowrap text-xs font-medium uppercase tracking-wide text-muted-foreground [&:has([role=checkbox])]:pr-0";
    let merged_class = tw_merge!(base, &class);

    view! {
        <th class=merged_class>
            {children()}
        </th>
    }
}

#[component]
pub fn TableCell(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "p-4 align-middle [&:has([role=checkbox])]:pr-0";
    let merged_class = tw_merge!(base, &class);

    view! {
        <td class=merged_class>
            {children()}
        </td>
    }
}

#[component]
pub fn TableCaption(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "mt-4 text-sm text-muted-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <caption class=merged_class>
            {children()}
        </caption>
    }
}
