//! Layer Showcase (AlertDialog, Sheet, Drawer, Popover, Tooltip, Toast, DeleteConfirmButton)

use std::time::Duration;

use leptos::prelude::*;
use web_package::ui::component::{
    AlertDialog, AlertDialogContent, AlertDialogDescription, AlertDialogFooter,
    AlertDialogHeader, AlertDialogTitle, Button, ButtonSize, ButtonVariant, Card, CardContent,
    CardDescription, CardHeader, CardTitle, DeleteConfirmButton, Drawer, DrawerDescription,
    DrawerFooter, DrawerHeader, DrawerSide, DrawerTitle, Popover, PopoverContent, PopoverTrigger,
    Sheet, SheetDescription, SheetFooter, SheetHeader, SheetSide, SheetTitle, Toast, Tooltip,
};

#[component]
pub fn LayerShowcase() -> impl IntoView {
    let (alert_open, set_alert_open) = signal(false);
    let (sheet_side, set_sheet_side) = signal(None::<SheetSide>);
    let (drawer_side, set_drawer_side) = signal(None::<DrawerSide>);
    let (pop_open, set_pop_open) = signal(false);
    let (toasts, set_toasts) = signal(Vec::<(u32, bool)>::new());
    let (next_id, set_next_id) = signal(0u32);
    let (deleted, set_deleted) = signal(0u32);

    let push_toast = move |is_error: bool| {
        let id = next_id.get_untracked();
        set_next_id.set(id + 1);
        set_toasts.update(|t| t.push((id, is_error)));
        set_timeout(
            move || set_toasts.update(|t| t.retain(|(i, _)| *i != id)),
            Duration::from_secs(4),
        );
    };

    let sheet_btn = move |label: &'static str, side: SheetSide| {
        view! {
            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline
                on_click=Callback::new(move |_| set_sheet_side.set(Some(side)))>{label}</Button>
        }
    };
    let drawer_btn = move |label: &'static str, side: DrawerSide| {
        view! {
            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline
                on_click=Callback::new(move |_| set_drawer_side.set(Some(side)))>{label}</Button>
        }
    };

    view! {
        <Card class="shadow-sm border-border">
            <CardHeader class="pb-3">
                <CardTitle class="text-base">"7. เลเยอร์ลอย (AlertDialog, Sheet, Drawer, Popover, Tooltip, Toast)"</CardTitle>
                <CardDescription>"คอมโพเนนต์ที่ลอยเหนือหน้าจอ และปุ่มยืนยันการลบ"</CardDescription>
            </CardHeader>
            <CardContent class="space-y-6">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"AlertDialog (ยืนยันการกระทำสำคัญ)"</h4>
                        <Button variant=ButtonVariant::Destructive on_click=Callback::new(move |_| set_alert_open.set(true))>
                            "เปิด AlertDialog"
                        </Button>
                        <AlertDialog open=Signal::derive(move || alert_open.get())>
                            <AlertDialogContent>
                                <AlertDialogHeader>
                                    <AlertDialogTitle>"ลบข้อมูลถาวร?"</AlertDialogTitle>
                                    <AlertDialogDescription>"การกระทำนี้ย้อนกลับไม่ได้ ต้องกดเลือกเพื่อปิดหน้าต่าง"</AlertDialogDescription>
                                </AlertDialogHeader>
                                <AlertDialogFooter>
                                    <Button variant=ButtonVariant::Outline on_click=Callback::new(move |_| set_alert_open.set(false))>"ยกเลิก"</Button>
                                    <Button variant=ButtonVariant::Destructive on_click=Callback::new(move |_| set_alert_open.set(false))>"ลบ"</Button>
                                </AlertDialogFooter>
                            </AlertDialogContent>
                        </AlertDialog>
                    </div>

                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"DeleteConfirmButton (กดสองครั้งเพื่อลบ)"</h4>
                        <div class="flex items-center gap-3">
                            <DeleteConfirmButton
                                label="ลบข้อมูล"
                                confirm_text="ยืนยันลบ?"
                                on_confirm=Callback::new(move |_| set_deleted.update(|d| *d += 1))
                            />
                            <span class="text-xs text-muted-foreground">{move || format!("ยืนยันลบแล้ว {} ครั้ง", deleted.get())}</span>
                        </div>
                    </div>
                </div>

                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"Sheet (แผงเลื่อนจากขอบจอ)"</h4>
                    <div class="flex flex-wrap gap-2">
                        {sheet_btn("Right", SheetSide::Right)}
                        {sheet_btn("Left", SheetSide::Left)}
                        {sheet_btn("Top", SheetSide::Top)}
                        {sheet_btn("Bottom", SheetSide::Bottom)}
                    </div>
                    {[SheetSide::Right, SheetSide::Left, SheetSide::Top, SheetSide::Bottom].into_iter().map(|side| view! {
                        <Sheet
                            open=Signal::derive(move || sheet_side.get() == Some(side))
                            on_close=Callback::new(move |_| set_sheet_side.set(None))
                            side=side
                        >
                            <SheetHeader>
                                <SheetTitle>{format!("Sheet ({side:?})")}</SheetTitle>
                                <SheetDescription>"แผงข้างสำหรับฟอร์มหรือรายละเอียดเพิ่มเติม คลิกฉากหลังเพื่อปิด"</SheetDescription>
                            </SheetHeader>
                            <SheetFooter>
                                <Button on_click=Callback::new(move |_| set_sheet_side.set(None))>"ปิด"</Button>
                            </SheetFooter>
                        </Sheet>
                    }).collect_view()}
                </div>

                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"Drawer (ลิ้นชักเลื่อนจากขอบจอ)"</h4>
                    <div class="flex flex-wrap gap-2">
                        {drawer_btn("Bottom", DrawerSide::Bottom)}
                        {drawer_btn("Top", DrawerSide::Top)}
                        {drawer_btn("Left", DrawerSide::Left)}
                        {drawer_btn("Right", DrawerSide::Right)}
                    </div>
                    {[DrawerSide::Bottom, DrawerSide::Top, DrawerSide::Left, DrawerSide::Right].into_iter().map(|side| view! {
                        <Drawer
                            open=Signal::derive(move || drawer_side.get() == Some(side))
                            on_close=Callback::new(move |_| set_drawer_side.set(None))
                            side=side
                        >
                            <DrawerHeader>
                                <DrawerTitle>{format!("Drawer ({side:?})")}</DrawerTitle>
                                <DrawerDescription>"ลิ้นชักสำหรับเนื้อหารอง คลิกฉากหลังเพื่อปิด"</DrawerDescription>
                            </DrawerHeader>
                            <DrawerFooter>
                                <Button on_click=Callback::new(move |_| set_drawer_side.set(None))>"ปิด"</Button>
                            </DrawerFooter>
                        </Drawer>
                    }).collect_view()}
                </div>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"Popover"</h4>
                        <Popover open=Signal::derive(move || pop_open.get())>
                            <PopoverTrigger on_toggle=Callback::new(move |_| set_pop_open.update(|o| *o = !*o))>
                                <span class="text-xs font-medium border border-border rounded-md px-3 py-1.5">"เปิด Popover"</span>
                            </PopoverTrigger>
                            <PopoverContent>
                                <p class="text-xs text-muted-foreground">"เนื้อหาลอยที่ผูกกับปุ่ม ควบคุมด้วย Signal"</p>
                            </PopoverContent>
                        </Popover>
                    </div>

                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"Tooltip (เลื่อนเมาส์ชี้)"</h4>
                        <Tooltip content="คำอธิบายเพิ่มเติม">
                            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline>"Hover me"</Button>
                        </Tooltip>
                    </div>

                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"Toast (หายเองใน 4 วินาที)"</h4>
                        <div class="flex gap-2">
                            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline on_click=Callback::new(move |_| push_toast(false))>"Success"</Button>
                            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline on_click=Callback::new(move |_| push_toast(true))>"Error"</Button>
                        </div>
                    </div>
                </div>

                <div class="fixed bottom-4 right-4 z-[60] flex w-80 max-w-[calc(100vw-2rem)] flex-col gap-2 pointer-events-none">
                    <For each=move || toasts.get() key=|(id, _)| *id
                        children=move |(id, is_error)| view! {
                            <Toast
                                title=if is_error { "เกิดข้อผิดพลาด" } else { "บันทึกสำเร็จ" }
                                description=if is_error { "ไม่สามารถทำรายการได้" } else { "ข้อมูลถูกบันทึกแล้ว" }
                                is_error=is_error
                                on_dismiss=Callback::new(move |_| set_toasts.update(|t| t.retain(|(i, _)| *i != id)))
                            />
                        } />
                </div>
            </CardContent>
        </Card>
    }
}
