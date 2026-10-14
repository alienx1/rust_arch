//! Navigation & Layout Showcase (Breadcrumb, Pagination, Stepper, Collapsible, ScrollArea, AspectRatio)

use leptos::prelude::*;
use web_package::ui::component::{
    AspectRatio, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, ButtonSize, ButtonVariant, Card, CardContent, CardDescription, CardHeader, CardTitle,
    Collapsible, CollapsibleContent, CollapsibleTrigger, Pagination, PaginationContent,
    PaginationEllipsis, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious,
    ScrollArea, Step, Stepper,
};

#[component]
pub fn NavigationShowcase() -> impl IntoView {
    let (open, set_open) = signal(false);
    let (step, set_step) = signal(2usize);

    view! {
        <Card class="shadow-sm border-border">
            <CardHeader class="pb-3">
                <CardTitle class="text-base">"6. การนำทางและเลย์เอาต์ (Navigation & Layout)"</CardTitle>
                <CardDescription>"Breadcrumb, Pagination, Stepper, Collapsible, ScrollArea และ AspectRatio"</CardDescription>
            </CardHeader>
            <CardContent class="space-y-6">
                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"Breadcrumb (เส้นทางนำทาง)"</h4>
                    <Breadcrumb>
                        <BreadcrumbItem><BreadcrumbLink href="/">"หน้าหลัก"</BreadcrumbLink></BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem><BreadcrumbLink href="/demo">"คอมโพเนนต์"</BreadcrumbLink></BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem><BreadcrumbPage>"Breadcrumb"</BreadcrumbPage></BreadcrumbItem>
                    </Breadcrumb>
                </div>

                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"Pagination (ตัวแบ่งหน้าแบบประกอบเอง)"</h4>
                    <Pagination>
                        <PaginationContent>
                            <PaginationItem><PaginationPrevious href="#" /></PaginationItem>
                            <PaginationItem><PaginationLink href="#">"1"</PaginationLink></PaginationItem>
                            <PaginationItem><PaginationLink href="#" is_active=true>"2"</PaginationLink></PaginationItem>
                            <PaginationItem><PaginationLink href="#">"3"</PaginationLink></PaginationItem>
                            <PaginationItem><PaginationEllipsis /></PaginationItem>
                            <PaginationItem><PaginationNext href="#" /></PaginationItem>
                        </PaginationContent>
                    </Pagination>
                </div>

                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"Stepper (ขั้นตอน)"</h4>
                    <div class="space-y-3">
                        {move || {
                            let current = step.get();
                            view! {
                                <Stepper>
                                    <Step step_number=1 title="ข้อมูลองค์กร" description="ชื่อและรหัส" is_completed={current > 1} is_current={current == 1} />
                                    <Step step_number=2 title="ผู้รับผิดชอบ" description="เลือกผู้ดูแล" is_completed={current > 2} is_current={current == 2} />
                                    <Step step_number=3 title="ยืนยัน" description="ตรวจสอบก่อนบันทึก" is_completed={current > 3} is_current={current == 3} />
                                </Stepper>
                            }
                        }}
                        <div class="flex gap-2">
                            <Button size=ButtonSize::Sm variant=ButtonVariant::Outline
                                on_click=Callback::new(move |_| set_step.update(|s| *s = (*s).saturating_sub(1).max(1)))>
                                "ย้อนกลับ"
                            </Button>
                            <Button size=ButtonSize::Sm
                                on_click=Callback::new(move |_| set_step.update(|s| *s = (*s + 1).min(4)))>
                                "ถัดไป"
                            </Button>
                        </div>
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"Collapsible"</h4>
                        <Collapsible open=Signal::derive(move || open.get())>
                            <CollapsibleTrigger on_toggle=Callback::new(move |_| set_open.update(|o| *o = !*o))>
                                <span class="text-xs font-medium">{move || if open.get() { "ซ่อนรายละเอียด ▲" } else { "แสดงรายละเอียด ▼" }}</span>
                            </CollapsibleTrigger>
                            <CollapsibleContent>
                                <div class="mt-2 p-3 bg-muted/40 rounded-lg text-xs text-muted-foreground">
                                    "เนื้อหาที่พับเก็บได้ ควบคุมด้วย Signal ภายนอก"
                                </div>
                            </CollapsibleContent>
                        </Collapsible>
                    </div>

                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"ScrollArea"</h4>
                        <ScrollArea class="h-32 rounded-md border border-border p-3">
                            {(1..=20).map(|i| view! {
                                <div class="text-xs py-1 border-b border-border last:border-0">{format!("รายการที่ {i}")}</div>
                            }).collect_view()}
                        </ScrollArea>
                    </div>

                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">"AspectRatio (16:9)"</h4>
                        <AspectRatio ratio=16.0 / 9.0 class="rounded-md bg-muted flex items-center justify-center text-xs text-muted-foreground">
                            "16 : 9"
                        </AspectRatio>
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
