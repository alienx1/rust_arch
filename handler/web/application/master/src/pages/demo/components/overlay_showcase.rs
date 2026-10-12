//! Overlays, Dialogs, Kbd, Separator & Empty State Showcase

use crate::pages::demo::providers::use_demo;
use leptos::prelude::*;
use web_package::ui::component::{
    Button, ButtonVariant, Card, CardContent, CardDescription, CardHeader, CardTitle, Dialog,
    DialogDescription, DialogFooter, DialogHeader, DialogTitle, Empty, EmptyActions,
    EmptyDescription, EmptyIcon, EmptyTitle, Kbd, Separator,
};

#[component]
pub fn OverlayShowcase() -> impl IntoView {
    let ctx = use_demo();
    let (dialog_open, set_dialog_open) = ctx.dialog_open;

    view! {
        <Card class="shadow-sm border-border">
            <CardHeader class="pb-3">
                <CardTitle class="text-base">"5. โอเวอร์เลย์และคอมโพเนนต์เสริม (Overlays, Modals, Kbd & Empty)"</CardTitle>
                <CardDescription>"คอมโพเนนต์ชั้นเลเยอร์บน Dialog, Keyboard Badges, Separator และ Empty State"</CardDescription>
            </CardHeader>
            <CardContent class="space-y-6">
                // Dialog Trigger & Modal
                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                        "Modal Dialog (หน้าต่างป๊อปอัป)"
                    </h4>
                    <div class="flex items-center gap-3">
                        <Button
                            variant=ButtonVariant::Default
                            on_click=Callback::new(move |_| set_dialog_open.set(true))
                        >
                            "เปิดหน้าต่าง Modal Dialog"
                        </Button>
                        <span class="text-xs text-muted-foreground">
                            "คลิกปุ่มเพื่อทดสอบการเปิด Dialog แบบมีฉากหลัง Backdrop"
                        </span>
                    </div>

                    // Interactive Dialog Component
                    <Dialog
                        open=Signal::derive(move || dialog_open.get())
                        on_close=Callback::new(move |()| set_dialog_open.set(false))
                    >
                        <DialogHeader>
                            <DialogTitle>"ยืนยันการทำรายการทดสอบ"</DialogTitle>
                            <DialogDescription>
                                "นี่คือหน้าต่าง Dialog ทดสอบของระบบ Master Data SPA ซึ่งทำงานผ่าน Leptos Signal State"
                            </DialogDescription>
                        </DialogHeader>
                        <div class="py-3 text-xs text-muted-foreground space-y-2">
                            <p>"สามารถใส่ฟอร์ม อินพุต หรือข้อความเตือนใดๆ ภายใน Dialog ได้อย่างอิสระ"</p>
                        </div>
                        <DialogFooter>
                            <Button
                                variant=ButtonVariant::Outline
                                on_click=Callback::new(move |_| set_dialog_open.set(false))
                            >
                                "ยกเลิก"
                            </Button>
                            <Button
                                variant=ButtonVariant::Default
                                on_click=Callback::new(move |_| set_dialog_open.set(false))
                            >
                                "ตกลง"
                            </Button>
                        </DialogFooter>
                    </Dialog>
                </div>

                <Separator />

                // Kbd & Separator
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "Kbd (ปุ่มคีย์ลัดบนแป้นพิมพ์)"
                        </h4>
                        <div class="flex items-center gap-2 text-xs text-foreground">
                            <span>"กด"</span>
                            <Kbd>"⌘"</Kbd>
                            <span>"+"</span>
                            <Kbd>"K"</Kbd>
                            <span>"เพื่อเปิดกล่องค้นหาด่วน หรือกด"</span>
                            <Kbd>"Esc"</Kbd>
                            <span>"เพื่อปิด"</span>
                        </div>
                    </div>

                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "Separator (เส้นคั่นแนวตั้ง / แนวนอน)"
                        </h4>
                        <div class="flex h-5 items-center space-x-4 text-xs">
                            <span>"Clean Architecture"</span>
                            <Separator orientation="vertical" />
                            <span>"CQRS"</span>
                            <Separator orientation="vertical" />
                            <span>"Rust 2024"</span>
                        </div>
                    </div>
                </div>

                <Separator />

                // Empty State
                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                        "Empty State (หน้าว่างเมื่อไม่มีข้อมูล)"
                    </h4>
                    <Empty class="min-h-[220px] p-6 bg-muted/20">
                        <EmptyIcon>
                            <svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5m8.25 3v6.75m0 0l-3-3m3 3l3-3M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
                            </svg>
                        </EmptyIcon>
                        <EmptyTitle>"ยังไม่มีข้อมูลรายงาน"</EmptyTitle>
                        <EmptyDescription>
                            "ยังไม่พบรายการข้อมูลในระบบ กรุณาคลิกปุ่มด้านล่างเพื่อเริ่มสร้างข้อมูลใหม่"
                        </EmptyDescription>
                        <EmptyActions>
                            <Button variant=ButtonVariant::Default>
                                "สร้างรายการแรก"
                            </Button>
                        </EmptyActions>
                    </Empty>
                </div>
            </CardContent>
        </Card>
    }
}
