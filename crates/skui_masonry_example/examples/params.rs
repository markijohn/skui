//#![cfg_attr(not(test), windows_subsystem = "windows")]

use masonry::core::{ErasedAction, NewWidget, Properties, Widget, WidgetId, WidgetTag};
use masonry::dpi::LogicalSize;
use masonry::layout::Length;
use masonry::peniko::color::AlphaColor;
use masonry::properties::Padding;
use masonry::properties::types::CrossAxisAlignment;
use masonry::theme::default_property_set;
use masonry::widgets::{Button, ButtonPress, Flex, Label, Portal, TextAction, TextArea, TextInput};
use masonry_testing::TestHarness;
use masonry_winit::app::{AppDriver, DriverCtx, NewWindow, WindowId};
use masonry_winit::winit::window::Window;
use skui::{Parameters, SKUIParseError, TokenAndSpan, Value, SKUI};
//mod builder;
use skui_masonry_example::{BasicWidgetBuilder, DefaultWidgetBuilder, RootWidgetBuilder};
use skui_masonry_example::params::ParamsStack;

struct Driver {
    window_id: WindowId,
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
    }
}

/// Return initial to-do-list without items.
pub fn make_widget_tree() -> NewWidget<impl Widget + ?Sized> {
    let src = r#"
MyButton1 :
    Flex(Horizontal) {
        Button( ${one} )
        Button( ${two} )
    }

MyButton2 :
    Flex(Horizontal) {
        Button( ${0.some} )
        Button( ${1.name} )
    }

MyButtonFromApp :
    Flex(Horizontal) {
        Label( "This button text is from the app" )
        Label( ${0.title} )
        Button( ${0.button_text} )
    }

Main:
    Flex(Vertical) .mystyle {
        Label( "Hello World!" )
        MyButton1( one="111", two="222" )
        MyButton2( {some="Sometext"}, {name="Name Text"} )
        MyButtonFromApp( ${0} )
    }

    "#;
    build_widget( src )
}

fn build_widget(src:&str) -> NewWidget<impl Widget + ?Sized> {
    let tks = TokenAndSpan::new(src);
    match SKUI::parse(&tks) {
        Ok(skui) => {
            let map = Value::Map(
                [("title", Value::String("Title from App")),
                 ("button_text", Value::String("My Custom Button!"))].into()
            );
            let args = vec![ map ];
            let parameters = Parameters::Args( args );

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

fn main() {
    let window_size = LogicalSize::new(400.0, 400.0);
    let window_attributes = Window::default_attributes()
        .with_title("To-do list")
        .with_resizable(true)
        .with_min_inner_size(window_size);
    let driver = Driver {
        window_id: WindowId::next(),
    };
    let event_loop = masonry_winit::app::EventLoop::with_user_event()
        .build()
        .unwrap();
    masonry_winit::app::run_with(
        event_loop,
        vec![
            NewWindow::new_with_id(
                driver.window_id,
                window_attributes,
                make_widget_tree().erased(),
            )
                .with_base_color(AlphaColor::from_rgb8(2, 6, 23)),
        ],
        driver,
        default_property_set(),
    )
        .unwrap();
}