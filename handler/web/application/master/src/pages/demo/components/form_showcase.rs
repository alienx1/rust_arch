//! Form Controls Component Showcase

use crate::pages::demo::providers::use_demo;
use leptos::prelude::*;
use web_package::ui::component::{
    Card, CardContent, CardDescription, CardHeader, CardTitle, Checkbox, DateRangePicker, Input,
    Label, RadioGroup, RadioGroupItem, SearchInput, Select, SelectOption, Slider, Switch, Textarea,
};

#[component]
pub fn FormShowcase() -> impl IntoView {
    let ctx = use_demo();
    let (checkbox_read, checkbox_write) = ctx.checkbox_val;
    let (switch_read, switch_write) = ctx.switch_val;
    let (progress_read, progress_write) = ctx.progress_val;
    let (input_read, input_write) = ctx.input_val;
    let (select_val, set_select_val) = signal("th".to_string());
    let (radio_val, set_radio_val) = signal("monthly".to_string());
    let (range, set_range) = signal((String::new(), String::new()));

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

                // Label, Select, Radio, DateRangePicker
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 pt-2">
                    <div class="space-y-1.5">
                        <Label r#for="demo-select">"Label + Select (ดรอปดาวน์)"</Label>
                        <Select
                            value=Signal::derive(move || select_val.get())
                            on_change=Callback::new(move |v| set_select_val.set(v))
                        >
                            <SelectOption value="th">"ประเทศไทย"</SelectOption>
                            <SelectOption value="jp">"ญี่ปุ่น"</SelectOption>
                            <SelectOption value="sg">"สิงคโปร์"</SelectOption>
                            <SelectOption value="xx" disabled=true>"ปิดใช้งาน"</SelectOption>
                        </Select>
                        <span class="text-[10px] text-muted-foreground block">{move || format!("เลือก: {}", select_val.get())}</span>
                    </div>

                    <div class="space-y-2">
                        <Label>"Radio Group"</Label>
                        <RadioGroup
                            value=Signal::derive(move || radio_val.get())
                            on_change=Callback::new(move |v| set_radio_val.set(v))
                        >
                            <RadioGroupItem value="daily" label="รายวัน" />
                            <RadioGroupItem value="monthly" label="รายเดือน" />
                            <RadioGroupItem value="never" label="ปิดใช้งาน" disabled=true />
                        </RadioGroup>
                        <span class="text-[10px] text-muted-foreground block">{move || format!("เลือก: {}", radio_val.get())}</span>
                    </div>

                    <div class="space-y-1.5 lg:col-span-2">
                        <Label>"DateRangePicker (ช่วงวันที่)"</Label>
                        <DateRangePicker
                            from=Signal::derive(move || range.get().0)
                            to=Signal::derive(move || range.get().1)
                            on_change=Callback::new(move |r| set_range.set(r))
                            aria_label="ช่วงวันที่"
                        />
                        <span class="text-[10px] text-muted-foreground block font-mono">
                            {move || { let (f, t) = range.get(); format!("{} → {}", if f.is_empty() { "-" } else { &f }, if t.is_empty() { "-" } else { &t }) }}
                        </span>
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
