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
    let dock = gtk::Box::new(gtk::Orientation::Vertical, 4);
    dock.set_widget_name("thumbnail-controls");
    dock.set_halign(gtk::Align::Center);
    dock.set_valign(gtk::Align::End);
    dock.set_margin_bottom(4);
    dock.append(&strip);
    dock.append(&button);
    dock.set_visible(available);
    overlay.add_overlay(&dock);
    overlay.set_measure_overlay(&dock, false);
    overlay.set_clip_overlay(&dock, true);
    let panel = strip.downgrade();
    let container = dock.downgrade();
    button.connect_toggled(move |button| {
        let (Some(panel), Some(container)) = (panel.upgrade(), container.upgrade()) else {
            return;
        };
        let open = button.is_active();
        panel.set_visible(open);
        container.set_halign(if open {
            gtk::Align::Fill
        } else {
            gtk::Align::Center
        });
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
