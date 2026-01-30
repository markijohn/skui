//#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::path::PathBuf;
use masonry::core::{ErasedAction, NewWidget, Properties, Widget, WidgetId, WidgetTag};
use masonry::dpi::LogicalSize;
use masonry::layout::Length;
use masonry::peniko::color::AlphaColor;
use masonry::properties::Padding;
use masonry::properties::types::CrossAxisAlignment;
use masonry::theme::default_property_set;
use masonry::widgets::{Button, ButtonPress, Flex, Label, Portal, SizedBox, TextAction, TextArea, TextInput};
use masonry_testing::TestHarness;
use masonry_winit::app::{AppDriver, DriverCtx, MasonryUserEvent, NewWindow, WindowId};
use masonry_winit::winit::window::Window;
use skui::{Parameters, SKUIParseError, TokenAndSpan, SKUI};
//mod builder;
use skui_masonry_example::{BasicWidgetBuilder, DefaultWidgetBuilder, RootWidgetBuilder};
use skui_masonry_example::params::ParamsStack;

const ROOT_WIDGET: WidgetTag<SizedBox> = WidgetTag::named("root");

struct Driver {
    next_task: String,
    window_id: WindowId,
    widget_id: WidgetId,
}

#[derive(PartialEq, Debug)]
pub enum FileChanged {
    Changed(u64),
}



impl AppDriver for Driver {
    fn on_action(
        &mut self,
        window_id: WindowId,
        ctx: &mut DriverCtx<'_, '_>,
        _widget_id: WidgetId,
        action: ErasedAction,
    ) {
        debug_assert_eq!(window_id, self.window_id, "unknown window");
        if action.is::<FileChanged>() {
            ctx.render_root(window_id).edit_widget_with_tag(ROOT_WIDGET, |mut root| {
                if let Ok(src) = std::fs::read_to_string(file_path()) {
                    // TODO: How dispose tag?
                    SizedBox::remove_child(&mut root);
                    let widget = build_widget( &src );
                    SizedBox::set_child(&mut root, widget);
                }
            });
        }
    }
}

/// Return initial to-do-list without items.
pub fn make_widget_tree() -> NewWidget<impl Widget + ?Sized> {
    NewWidget::new_with_tag(
        SizedBox::new( NewWidget::new((Label::new("None"))) )
    ,ROOT_WIDGET)
}

fn build_widget(src:&str) -> NewWidget<impl Widget + ?Sized> {
    let tks = TokenAndSpan::new(src);
    match SKUI::parse(&tks) {
        Ok(skui) => {
            let parameters = Parameters::empty();
            let Some(params_stack) = ParamsStack::new_main(&parameters, &skui)
            else { return NewWidget::new( Label::new( "Can't find Main component." ) ).erased() };
            match BasicWidgetBuilder::build_widget( &params_stack ) {
                Ok(widget) => widget.erased(),
                Err(e) => NewWidget::new( Label::new( format!("{e:#?}") ) ).erased()
            }
        }
        Err( e ) => {
            let text = format!("{e:#?}\n{}", tks.render_error_from_span(src, e.span.clone(),3));
            NewWidget::new( Label::new( text ) ).erased()
        }
    }
}

fn file_path() -> PathBuf {
    format!("{}/skui.txt", env!("CARGO_MANIFEST_DIR")).into()
}

fn main() {
    let window_size = LogicalSize::new(400.0, 400.0);
    let window_attributes = Window::default_attributes()
        .with_title("To-do list")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    let root_widget = make_widget_tree();
    let root_widget_id = root_widget.id();

    let driver = Driver {
        next_task: String::new(),
        window_id: WindowId::next(),
        widget_id: root_widget_id
    };
    let event_loop = masonry_winit::app::EventLoop::with_user_event()
        .build()
        .unwrap();
    let proxy = event_loop.create_proxy();

    const FILE_CHECK_INTERVAL:u64 = 50;
    std::thread::spawn(move || {
        let file_path = file_path();
        let mut last_file_changed = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(FILE_CHECK_INTERVAL));
            if let Ok(Ok(modified)) = std::fs::metadata(&file_path).map(|meta| meta.modified()) {
                let curr = modified.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
                if last_file_changed == curr { continue };
                last_file_changed = curr;
                if let Err(e) = proxy.send_event( MasonryUserEvent::Action(driver.window_id, Box::new(FileChanged::Changed(0)), root_widget_id ) ) {
                    println!("Send failed : {e}");
                    break;
                }
            } else { continue };
        }
    });
    masonry_winit::app::run_with(
        event_loop,
        vec![
            NewWindow::new_with_id(
                driver.window_id,
                window_attributes,
                root_widget.erased(),
            )
                .with_base_color(AlphaColor::from_rgb8(2, 6, 23)),
        ],
        driver,
        default_property_set(),
    )
        .unwrap();
}