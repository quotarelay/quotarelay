pub struct Widget {
    pub id: usize,
}

pub fn render_widget(widget: &Widget) -> String {
    format!("widget-{}", widget.id)
}

pub fn needle_workflow() -> &'static str {
    "needle workflow context"
}
