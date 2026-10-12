//! Button & Action Component Showcase

use crate::pages::demo::providers::use_demo;
use leptos::prelude::*;
use web_package::ui::component::{
    Button, ButtonGroup, ButtonSize, ButtonVariant, Card, CardContent, CardDescription, CardHeader,
    CardTitle, Spinner,
};

#[component]
pub fn ButtonShowcase() -> impl IntoView {
    let ctx = use_demo();
    let (counter_read, counter_write) = ctx.counter;

    view! {
        <Card class="shadow-sm border-border">
            <CardHeader class="pb-3">
                <CardTitle class="text-base">"1. ปุ่มและการกระทำ (Buttons & Action Variants)"</CardTitle>
                <CardDescription>"รวมรูปแบบปุ่มกดทั้งหมด (Variants, Sizes, States, ButtonGroup)"</CardDescription>
            </CardHeader>
            <CardContent class="space-y-6">
                // Variants
                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                        "Variants (รูปแบบของปุ่ม)"
                    </h4>
                    <div class="flex flex-wrap gap-2.5 items-center">
                        <Button variant=ButtonVariant::Default>
                            "Default Button"
                        </Button>
                        <Button variant=ButtonVariant::Secondary>
                            "Secondary Button"
                        </Button>
                        <Button variant=ButtonVariant::Outline>
                            "Outline Button"
                        </Button>
                        <Button variant=ButtonVariant::Destructive>
                            "Destructive Button"
                        </Button>
                        <Button variant=ButtonVariant::Ghost>
                            "Ghost Button"
                        </Button>
                        <Button variant=ButtonVariant::Link>
                            "Link Button"
                        </Button>
                    </div>
                </div>

                // Sizes
                <div>
                    <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                        "Sizes (ขนาดของปุ่ม)"
                    </h4>
                    <div class="flex flex-wrap gap-2.5 items-center">
                        <Button size=ButtonSize::Sm variant=ButtonVariant::Outline>
                            "Small (Sm)"
                        </Button>
                        <Button size=ButtonSize::Default variant=ButtonVariant::Outline>
                            "Default Size"
                        </Button>
                        <Button size=ButtonSize::Lg variant=ButtonVariant::Outline>
                            "Large (Lg)"
                        </Button>
                        <Button size=ButtonSize::Icon variant=ButtonVariant::Outline attr:title="Icon Button">
                            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path d="M12 5v14M5 12h14" />
                            </svg>
                        </Button>
                    </div>
                </div>

                // States & ButtonGroup
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "States & Interactive Click"
                        </h4>
                        <div class="flex flex-wrap gap-2.5 items-center">
                            <Button disabled=true variant=ButtonVariant::Default>
                                "Disabled"
                            </Button>
                            <Button variant=ButtonVariant::Outline class="gap-1.5" disabled=true>
                                <Spinner class="h-3.5 w-3.5 text-primary" />
                                "Loading..."
                            </Button>
                            <Button
                                variant=ButtonVariant::Default
                                on_click=Callback::new(move |_| counter_write.update(|c| *c += 1))
                            >
                                {move || format!("คลิกทดสอบ ({})", counter_read.get())}
                            </Button>
                        </div>
                    </div>

                    <div>
                        <h4 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                            "Button Group (กลุ่มปุ่มเชื่อมโยง)"
                        </h4>
                        <ButtonGroup>
                            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                                "ย้อนกลับ"
                            </Button>
                            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                                "ปัจจุบัน"
                            </Button>
                            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>
                                "ถัดไป"
                            </Button>
                        </ButtonGroup>
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
