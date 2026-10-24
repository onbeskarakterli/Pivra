//! TH-02 layout only. No provider queries, catalog selection, decoder or settings.
use gtk::prelude::*;
use std::cell::Cell;
thread_local! { static CSS_INSTALLED: Cell<bool> = const { Cell::new(false) }; }
pub(crate) const CELL_WIDTH: i32 = 96;
pub(crate) const CELL_HEIGHT: i32 = 80;
fn install_css(display: &gtk::gdk::Display) {
    CSS_INSTALLED.with(|installed| {
        if installed.replace(true) { return; }
        let css=gtk::CssProvider::new();
        css.load_from_string("\
            .pivra-thumbnail-panel { background-color: alpha(@window_bg_color, 0.25); border-radius: 12px; padding: 8px; }\n\
            .pivra-thumbnail-toggle { background-image: none; background-color: alpha(@window_bg_color, 0.25); border-radius: 16px; padding: 4px; min-width: 24px; min-height: 24px; }\n\
            .pivra-thumbnail-cell { background-color: transparent; border: none; border-radius: 0; box-shadow: none; }\n\
            .pivra-thumbnail-cell.selected { box-shadow: inset 0 -1px 0 0 @accent_color; }\n");
        gtk::style_context_add_provider_for_display(display,&css,gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    });
}
/// Paint-only underline never changes selected/unselected allocation.
#[allow(dead_code)] // Cell content is supplied only by the explicit TH-02 probe for now.
pub(crate) fn cell(child: &impl IsA<gtk::Widget>, selected: bool) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.add_css_class("pivra-thumbnail-cell");
    if selected {
        frame.add_css_class("selected");
    }
    frame.set_size_request(CELL_WIDTH, CELL_HEIGHT);
    frame.set_valign(gtk::Align::Center);
    // Overlay content cannot enlarge the fixed cell's natural request.
    let content = gtk::Overlay::new();
    content.add_overlay(child);
    content.set_measure_overlay(child, false);
    content.set_clip_overlay(child, true);
    frame.set_child(Some(&content));
    frame
}
pub(crate) fn install(overlay: &gtk::Overlay, available: bool) {
    install_css(&overlay.display());
    let items = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    items.set_widget_name("thumbnail-items");
    items.append(&gtk::Label::new(Some("No thumbnails available")));
    let strip = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .propagate_natural_width(false)
        .height_request(112)
        .child(&items)
        .build();
    strip.set_widget_name("thumbnail-strip");
    strip.add_css_class("pivra-thumbnail-panel");
    strip.set_margin_start(8);
    strip.set_margin_end(8);
    strip.set_visible(false);
    let button = gtk::ToggleButton::builder()
        .icon_name("pan-up-symbolic")
        .tooltip_text("Show thumbnail strip")
        .halign(gtk::Align::Center)
        .build();
    button.set_widget_name("thumbnail-toggle");
    button.add_css_class("pivra-thumbnail-toggle");
    // Independent overlay siblings: no full-width box beneath the strip that
    // can paint a dock background or intercept input around the small button.
    button.set_valign(gtk::Align::End);
    button.set_margin_bottom(4);
    button.set_visible(available);
    strip.set_valign(gtk::Align::End);
    // Reserve only overlay spacing, never space in the image's allocation.
    strip.set_margin_bottom(button.measure(gtk::Orientation::Vertical, -1).1 + 8);
    for widget in [strip.upcast_ref::<gtk::Widget>(), button.upcast_ref()] {
        overlay.add_overlay(widget);
        overlay.set_measure_overlay(widget, false);
        overlay.set_clip_overlay(widget, true);
    }
    let panel = strip.downgrade();
    button.connect_toggled(move |button| {
        let Some(panel) = panel.upgrade() else {
            return;
        };
        let open = button.is_active();
        panel.set_visible(open);
        button.set_icon_name(if open {
            "pan-down-symbolic"
        } else {
            "pan-up-symbolic"
        });
        button.set_tooltip_text(Some(if open {
            "Hide thumbnail strip"
        } else {
            "Show thumbnail strip"
        }));
    });
    // Widget ownership remains with Overlay; signal captures only weak references.
}
