//! Data Display Component Showcase (Card, Table, Accordion, Avatar, Tabs)

use crate::pages::demo::services::DemoMockService;
use leptos::prelude::*;
use web_package::ui::component::{
    Accordion, AccordionContent, AccordionItem, AccordionTrigger, Avatar, Badge, BadgeVariant,
    Button, ButtonSize, ButtonVariant, Card, CardContent, CardDescription, CardFooter, CardHeader,
    CardTitle, DataColumn, DataTable, FilterChips, TableCell, TableRow, Tabs, TabsContent,
    TabsList, TabsTrigger, data_row_number,
};

/// Rows per page of the demo table
const DEMO_PAGE_SIZE: u64 = 5;

#[component]
pub fn DataShowcase() -> impl IntoView {
    let users = StoredValue::new(DemoMockService::get_mock_users());
    let (active_tab, set_active_tab) = signal("tab1".to_string());

    // Client-side search / status filter / paging for the DataTable demo
    let (search, set_search) = signal(String::new());
    let (status, set_status) = signal("ALL".to_string());
    let (page, set_page) = signal(1u64);

    let filtered = move || {
        let q = search.get().trim().to_lowercase();
        let st = status.get();
        users
            .get_value()
            .into_iter()
            .filter(|u| st == "ALL" || u.status == st)
            .filter(|u| {
                q.is_empty()
                    || u.name.to_lowercase().contains(&q)
                    || u.email.to_lowercase().contains(&q)
                    || u.department.to_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    };
    let page_items = move || {
        let start = ((page.get().max(1) - 1) * DEMO_PAGE_SIZE) as usize;
        filtered().into_iter().skip(start).take(DEMO_PAGE_SIZE as usize).collect::<Vec<_>>()
    };

    let columns = vec![
        DataColumn::new("#").class("w-12 text-center"),
        DataColumn::new("รหัส").class("w-24"),
        DataColumn::new("ผู้ใช้งาน (Avatar + Name)").class("min-w-[200px]"),
        DataColumn::new("บทบาท (Role)"),
        DataColumn::new("แผนกงาน"),
        DataColumn::new("สถานะ").class("w-28 text-center"),
        DataColumn::new("การกระทำ").class("w-28 text-right"),
    ];

    view! {
        <Card class="shadow-sm border-border">
            <CardHeader class="pb-3">
                <CardTitle class="text-base">"4. ตารางและการแสดงผลข้อมูล (Data Display, Tables, Cards & Accordion)"</CardTitle>
                <CardDescription>"คอมโพเนนต์สำหรับจัดวางและแสดงโครงสร้างข้อมูลในระบบ Enterprise"</CardDescription>
            </CardHeader>
            <CardContent class="space-y-6">
                // Table Showcase (shared DataTable: search, chips, client-side paging)
                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                        "Enterprise Table (DataTable: ค้นหา, ชิปกรอง, แบ่งหน้า — ตารางข้อมูลผู้ใช้งานจำลอง)"
                    </h4>
                    <DataTable
                        title="ผู้ใช้งานจำลอง (Mock Users)"
                        description="ตัวอย่างการใช้ DataTable จาก web_package: ค้นหา, กรองสถานะ และแบ่งหน้าฝั่งไคลเอนต์"
                        icon_path="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"
                        columns=columns
                        empty=Signal::derive(move || page_items().is_empty())
                        empty_title="ไม่พบผู้ใช้งาน"
                        empty_text="ไม่พบผู้ใช้งานที่ตรงกับเงื่อนไข"
                        search_value=Signal::derive(move || search.get())
                        search_placeholder="ค้นหาชื่อ, อีเมล, แผนก..."
                        on_search=Callback::new(move |v| {
                            set_page.set(1);
                            set_search.set(v);
                        })
                        filters=move || view! {
                            <FilterChips
                                label="สถานะ:"
                                options=vec![
                                    ("ALL".to_string(), "ทั้งหมด".to_string()),
                                    ("Active".to_string(), "Active".to_string()),
                                    ("Pending".to_string(), "Pending".to_string()),
                                    ("Inactive".to_string(), "Inactive".to_string()),
                                ]
                                value=Signal::derive(move || status.get())
                                on_change=Callback::new(move |v| {
                                    set_page.set(1);
                                    set_status.set(v);
                                })
                            />
                        }
                        page=page
                        limit=DEMO_PAGE_SIZE
                        total=Signal::derive(move || filtered().len() as u64)
                        on_page=Callback::new(move |p| set_page.set(p))
                    >
                        {move || page_items().into_iter().enumerate().map(|(idx, u)| {
                            let initial = u.name.chars().next().unwrap_or('U').to_string();
                            let badge_variant = match u.status {
                                "Active" => BadgeVariant::Success,
                                "Pending" => BadgeVariant::Warning,
                                _ => BadgeVariant::Secondary,
                            };
                            view! {
                                <TableRow class="hover:bg-muted/30 transition-colors">
                                    <TableCell class="text-center font-mono text-xs text-muted-foreground">
                                        {data_row_number(page.get_untracked(), DEMO_PAGE_SIZE, idx)}
                                    </TableCell>
                                    <TableCell class="font-mono text-xs font-medium text-muted-foreground">
                                        {u.id}
                                    </TableCell>
                                    <TableCell>
                                        <div class="flex items-center gap-3">
                                            <Avatar fallback=initial class="h-9 w-9 border border-primary/20 text-primary font-bold text-xs shrink-0" />
                                            <div class="min-w-0">
                                                <div class="font-semibold text-foreground text-xs">{u.name}</div>
                                                <div class="text-[11px] text-muted-foreground">{u.email}</div>
                                            </div>
                                        </div>
                                    </TableCell>
                                    <TableCell class="text-xs font-medium">{u.role}</TableCell>
                                    <TableCell class="text-xs text-muted-foreground">{u.department}</TableCell>
                                    <TableCell class="text-center">
                                        <Badge variant=badge_variant class="text-xs">{u.status}</Badge>
                                    </TableCell>
                                    <TableCell class="text-right">
                                        <Button size=ButtonSize::Sm variant=ButtonVariant::Ghost class="h-7 text-xs">
                                            "ดูข้อมูล"
                                        </Button>
                                    </TableCell>
                                </TableRow>
                            }
                        }).collect_view()}
                    </DataTable>
                </div>

                // Tabs and Accordion Grid
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
                    // Tabs
                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "Tabs (แท็บสลับข้อมูล)"
                        </h4>
                        <Tabs active_tab=Signal::derive(move || active_tab.get())>
                            <TabsList>
                                <TabsTrigger value="tab1".to_string() on_select=Callback::new(move |v| set_active_tab.set(v))>
                                    "ข้อมูลทั่วไป"
                                </TabsTrigger>
                                <TabsTrigger value="tab2".to_string() on_select=Callback::new(move |v| set_active_tab.set(v))>
                                    "ความปลอดภัย"
                                </TabsTrigger>
                                <TabsTrigger value="tab3".to_string() on_select=Callback::new(move |v| set_active_tab.set(v))>
                                    "การเชื่อมต่อ"
                                </TabsTrigger>
                            </TabsList>
                            <TabsContent value="tab1".to_string()>
                                <div class="p-3 bg-muted/40 rounded-lg text-xs text-muted-foreground">
                                    "เนื้อหาแท็บที่ 1: แสดงข้อมูลรายละเอียดทั่วไปของระบบ Master Data"
                                </div>
                            </TabsContent>
                            <TabsContent value="tab2".to_string()>
                                <div class="p-3 bg-muted/40 rounded-lg text-xs text-muted-foreground">
                                    "เนื้อหาแท็บที่ 2: ระบบยืนยันตัวตนผ่าน Ory Kratos 26.2 และ Session Cookie"
                                </div>
                            </TabsContent>
                            <TabsContent value="tab3".to_string()>
                                <div class="p-3 bg-muted/40 rounded-lg text-xs text-muted-foreground">
                                    "เนื้อหาแท็บที่ 3: สถาปัตยกรรม Active-Active Replication บน Spock pgEdge"
                                </div>
                            </TabsContent>
                        </Tabs>
                    </div>

                    // Accordion
                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "Accordion (แถบพับเก็บข้อมูล)"
                        </h4>
                        <Accordion>
                            <AccordionItem>
                                <AccordionTrigger>
                                    "Clean Architecture คืออะไร?"
                                </AccordionTrigger>
                                <AccordionContent>
                                    <p class="text-xs text-muted-foreground">
                                        "โครงสร้างสถาปัตยกรรมที่แยก Domain, Application, Infrastructure และ Handler ออกจากกันอย่างเคร่งครัด"
                                    </p>
                                </AccordionContent>
                            </AccordionItem>
                            <AccordionItem>
                                <AccordionTrigger>
                                    "ทำไมต้องใช้ Rust 2024 Edition?"
                                </AccordionTrigger>
                                <AccordionContent>
                                    <p class="text-xs text-muted-foreground">
                                        "เพื่อประสิทธิภาพระดับสูงสุด ประหยัดหน่วยความจำ และความปลอดภัยระดับ Type System"
                                    </p>
                                </AccordionContent>
                            </AccordionItem>
                        </Accordion>
                    </div>
                </div>

                // Card Structure Showcase
                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                        "Card with Header & Footer"
                    </h4>
                    <Card class="bg-card/50">
                        <CardHeader>
                            <CardTitle class="text-sm">"หัวข้อการ์ดมาตรฐาน (Card Header)"</CardTitle>
                            <CardDescription class="text-xs">"คำอธิบายสั้นๆ เกี่ยวกับเนื้อหาในการ์ด"</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <p class="text-xs text-muted-foreground">
                                "ส่วนเนื้อหาหลัก (CardContent) สำหรับบรรจุฟอร์ม สถิติ หรือคอมโพเนนต์ย่อยใดๆ ได้ตามต้องการ"
                            </p>
                        </CardContent>
                        <CardFooter class="flex justify-between items-center border-t border-border pt-3">
                            <span class="text-[11px] text-muted-foreground font-mono">"CardFooter v1.0"</span>
                            <Button size=ButtonSize::Sm variant=ButtonVariant::Default>
                                "บันทึกข้อมูล"
                            </Button>
                        </CardFooter>
                    </Card>
                </div>
            </CardContent>
        </Card>
    }
}
