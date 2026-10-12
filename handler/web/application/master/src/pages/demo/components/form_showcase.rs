//! Form Controls Component Showcase

use crate::pages::demo::providers::use_demo;
use leptos::prelude::*;
use web_package::ui::component::{
    Card, CardContent, CardDescription, CardHeader, CardTitle, Checkbox, Input, SearchInput,
    Slider, Switch, Textarea,
};

#[component]
pub fn FormShowcase() -> impl IntoView {
    let ctx = use_demo();
    let (checkbox_read, checkbox_write) = ctx.checkbox_val;
    let (switch_read, switch_write) = ctx.switch_val;
    let (progress_read, progress_write) = ctx.progress_val;
    let (input_read, input_write) = ctx.input_val;

    view! {
        <Card class="shadow-sm border-border">
            <CardHeader class="pb-3">
                <CardTitle class="text-base">"2. ฟอร์มและอินพุต (Form Controls & Input Fields)"</CardTitle>
                <CardDescription>"ชุดคอมโพเนนต์รับข้อมูล: Input, Search, Textarea, Checkbox, Switch, Slider"</CardDescription>
            </CardHeader>
            <CardContent class="space-y-6">
                // Text Inputs
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                        <label class="text-xs font-semibold text-foreground block mb-1.5">
                            "Standard Input"
                        </label>
                        <Input
                            placeholder="กรอกชื่อองค์กร..."
                            value=Signal::derive(move || input_read.get())
                            on_input=Callback::new(move |val| input_write.set(val))
                        />
                        <span class="text-[10px] text-muted-foreground mt-1 block">
                            {move || format!("ค่าปัจจุบัน: {}", input_read.get())}
                        </span>
                    </div>

                    <div>
                        <label class="text-xs font-semibold text-foreground block mb-1.5">
                            "Search Input (ค้นหา)"
                        </label>
                        <SearchInput placeholder="ค้นหาข้อมูล..." />
                    </div>

                    <div>
                        <label class="text-xs font-semibold text-foreground block mb-1.5">
                            "Disabled Input"
                        </label>
                        <Input
                            placeholder="ไม่สามารถแก้ไขได้"
                            disabled=true
                        />
                    </div>
                </div>

                // Textarea
                <div>
                    <label class="text-xs font-semibold text-foreground block mb-1.5">
                        "Textarea (กล่องข้อความหลายบรรทัด)"
                    </label>
                    <Textarea placeholder="ระบุรายละเอียดหรือหมายเหตุเพิ่มเติม..." rows=3 />
                </div>

                // Toggles & Selection
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6 pt-2">
                    // Checkbox
                    <div class="space-y-3">
                        <label class="text-xs font-semibold text-muted-foreground uppercase tracking-wider block">
                            "Checkbox"
                        </label>
                        <Checkbox
                            label="เปิดใช้งานระบบแจ้งเตือน"
                            checked=Signal::derive(move || checkbox_read.get())
                            on_change=Callback::new(move |val| checkbox_write.set(val))
                        />
                        <Checkbox
                            label="ตัวเลือกที่ปิดการใช้งาน"
                            checked=Signal::derive(|| false)
                            on_change=Callback::new(|_| ())
                            disabled=true
                        />
                    </div>

                    // Switch
                    <div class="space-y-3">
                        <label class="text-xs font-semibold text-muted-foreground uppercase tracking-wider block">
                            "Switch Toggle"
                        </label>
                        <Switch
                            label="เปิดการเชื่อมต่อ Active-Active"
                            checked=Signal::derive(move || switch_read.get())
                            on_change=Callback::new(move |val| switch_write.set(val))
                        />
                        <Switch
                            label="โหมดบำรุงรักษา (Disabled)"
                            checked=Signal::derive(|| false)
                            on_change=Callback::new(|_| ())
                            disabled=true
                        />
                    </div>

                    // Slider
                    <div>
                        <div class="flex items-center justify-between mb-2">
                            <label class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                                "Slider Control"
                            </label>
                            <span class="text-xs font-mono font-bold text-primary">
                                {move || format!("{:.0}%", progress_read.get())}
                            </span>
                        </div>
                        <Slider
                            value=Signal::derive(move || progress_read.get())
                            on_change=Callback::new(move |val| progress_write.set(val))
                            min=0.0
                            max=100.0
                        />
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
