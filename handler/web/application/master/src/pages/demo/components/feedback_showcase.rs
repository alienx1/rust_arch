//! Feedback, Badges, Alerts & Status Showcase

use crate::pages::demo::providers::use_demo;
use leptos::prelude::*;
use web_package::ui::component::{
    Alert, AlertDescription, AlertTitle, AlertVariant, Badge, BadgeVariant, Card, CardContent,
    CardDescription, CardHeader, CardTitle, Progress, Skeleton, Spinner, StatusColor, StatusDot,
};

#[component]
pub fn FeedbackShowcase() -> impl IntoView {
    let ctx = use_demo();
    let (progress_read, _) = ctx.progress_val;

    view! {
        <Card class="shadow-sm border-border">
            <CardHeader class="pb-3">
                <CardTitle class="text-base">"3. การแจ้งเตือนและสถานะ (Feedback, Alerts, Badges & Status)"</CardTitle>
                <CardDescription>"คอมโพเนนต์แสดงสถานะ ตอบสนองผู้ใช้ และตัวบ่งชี้ความคืบหน้า"</CardDescription>
            </CardHeader>
            <CardContent class="space-y-6">
                // Alerts
                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                        "Alerts (กล่องข้อความแจ้งเตือน)"
                    </h4>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <Alert variant=AlertVariant::Default>
                            <AlertTitle>"ข้อมูลทั่วไป (Default Alert)"</AlertTitle>
                            <AlertDescription>"ระบบกำลังทำงานอยู่ในโหมดมาตรฐาน สามารถทำรายการได้ตามปกติ"</AlertDescription>
                        </Alert>

                        <Alert variant=AlertVariant::Success>
                            <AlertTitle>"บันทึกสำเร็จ (Success Alert)"</AlertTitle>
                            <AlertDescription>"ข้อมูลโครงสร้างองค์กรถูกซิงค์ไปยังคลัสเตอร์เรียบร้อยแล้ว"</AlertDescription>
                        </Alert>

                        <Alert variant=AlertVariant::Warning>
                            <AlertTitle>"คำเตือน (Warning Alert)"</AlertTitle>
                            <AlertDescription>"กำลังใช้งานในสภาพแวดล้อม Local Development (Static Mode)"</AlertDescription>
                        </Alert>

                        <Alert variant=AlertVariant::Destructive>
                            <AlertTitle>"ข้อผิดพลาด (Destructive Alert)"</AlertTitle>
                            <AlertDescription>"ไม่สามารถเชื่อมต่อฐานข้อมูลได้ กรุณาตรวจสอบ Connection Pool"</AlertDescription>
                        </Alert>
                    </div>
                </div>

                // Badges & Status Dots
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "Badges (ป้ายกำกับ)"
                        </h4>
                        <div class="flex flex-wrap gap-2 items-center">
                            <Badge variant=BadgeVariant::Default>"Default"</Badge>
                            <Badge variant=BadgeVariant::Secondary>"Secondary"</Badge>
                            <Badge variant=BadgeVariant::Outline>"Outline"</Badge>
                            <Badge variant=BadgeVariant::Success>"Active / Success"</Badge>
                            <Badge variant=BadgeVariant::Warning>"Pending / Warning"</Badge>
                            <Badge variant=BadgeVariant::Destructive>"Error / Failed"</Badge>
                        </div>
                    </div>

                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "Status Dots (จุดบอกสถานะ)"
                        </h4>
                        <div class="flex flex-wrap gap-4 items-center">
                            <StatusDot color=StatusColor::Success label="Online (เขียว)" />
                            <StatusDot color=StatusColor::Warning label="Syncing (ส้ม)" />
                            <StatusDot color=StatusColor::Destructive label="Offline (แดง)" />
                            <StatusDot color=StatusColor::Info label="Standby (ฟ้า)" />
                            <StatusDot color=StatusColor::Muted label="Disabled (เทา)" />
                        </div>
                    </div>
                </div>

                // Progress, Spinners & Skeleton
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6 pt-2">
                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "Progress Bar (ความคืบหน้า)"
                        </h4>
                        <div class="space-y-2">
                            <Progress value=Signal::derive(move || progress_read.get()) />
                            <span class="text-[11px] text-muted-foreground block text-right font-mono">
                                {move || format!("{:.0}% เสร็จสมบูรณ์", progress_read.get())}
                            </span>
                        </div>
                    </div>

                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "Spinners (สัญลักษณ์โหลด)"
                        </h4>
                        <div class="flex items-center gap-4">
                            <Spinner class="h-4 w-4 text-primary" />
                            <Spinner class="h-6 w-6 text-primary" />
                            <Spinner class="h-8 w-8 text-primary" />
                            <span class="text-xs text-muted-foreground">"กำลังประมวลผล..."</span>
                        </div>
                    </div>

                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "Skeleton Placeholder"
                        </h4>
                        <div class="space-y-2">
                            <Skeleton class="h-4 w-full" />
                            <Skeleton class="h-4 w-[75%]" />
                            <Skeleton class="h-4 w-[50%]" />
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
