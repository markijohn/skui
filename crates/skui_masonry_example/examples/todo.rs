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
use skui::{render_error, SKUIParseError, TokensAndSpan, SKUI};
//mod builder;
use skui_masonry_example::{get_widget_tag, DefaultWidgetBuilder, RootWidgetBuilder};
use skui_masonry_example::params::ParamsStack;

const TEXT_INPUT_TAG: WidgetTag<TextInput> = WidgetTag::named("text_input");
const LIST_TAG: WidgetTag<Flex> = WidgetTag::named("list");
const WIDGET_SPACING: Length = Length::const_px(5.0);

struct Driver {
    next_task: String,
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
        // let input_tag = TEXT_INPUT_TAG;
        // let list_tag = LIST_TAG;
        let (input_tag, list_tag) = unsafe { (
            get_widget_tag::<TextInput>("text_input"),
            get_widget_tag::<Flex>("list")
        ) };

        debug_assert_eq!(window_id, self.window_id, "unknown window");
        if action.is::<ButtonPress>() {
            let render_root = ctx.render_root(window_id);

            render_root.edit_widget_with_tag(input_tag, |mut text_input| {
                let mut text_area = TextInput::text_mut(&mut text_input);
                TextArea::reset_text(&mut text_area, "");
            });
            render_root.edit_widget_with_tag(list_tag, |mut list| {
                let child = Label::new(self.next_task.clone()).with_auto_id();
                Flex::add_fixed(&mut list, child);
            });
        } else if action.is::<TextAction>() {
            let action = action.downcast::<TextAction>().unwrap();
            match *action {
                TextAction::Changed(new_text) => {
                    self.next_task = new_text.clone();
                }
                TextAction::Entered(_) => {}
            }
        }
    }
}

/// Return initial to-do-list without items.
pub fn make_widget_tree() -> NewWidget<impl Widget + ?Sized> {
    // origin source
    // let text_input = NewWidget::new_with_tag(
    //     TextInput::new("").with_placeholder("ex: 'Do the dishes', 'File my taxes', ..."),
    //     TEXT_INPUT_TAG,
    // );
    // let button = NewWidget::new(Button::with_text("Add task"));
    //
    // let portal = Portal::new(NewWidget::new_with_tag(
    //     Flex::column().cross_axis_alignment(CrossAxisAlignment::Start),
    //     LIST_TAG,
    // ))
    //     .with_auto_id();
    //
    // let root = Flex::column()
    //     .with_fixed(NewWidget::new_with_props(
    //         Flex::row().with(text_input, 1.0).with_fixed(button),
    //         Properties::new().with(Padding::all(WIDGET_SPACING.get())),
    //     ))
    //     .with_fixed_spacer(WIDGET_SPACING)
    //     .with(portal, 1.0);
    //
    // NewWidget::new(root)

    let src = r#"
    Main: Flex(Vertical) {
            Flex(Horizontal) {
                padding : 30.0
                FlexItem(TextInput("ex: 'Do the dishes', 'File my taxes', ...") #text_input , 1.0 )
                Button("Add task")
            }
            Spacing(1.0)
            FlexItem( Portal(
                Flex(axis=Vertical, cross_axis_alignment=Start) #list
            ), 1.0 )
        }
    "#;

    let tks = TokensAndSpan::new(src);
    match SKUI::parse(&tks) {
        Ok(skui) => {
            println!("{:#?}", skui.components);
            let Some(params_stack) = ParamsStack::new_main(&skui)
            else { return NewWidget::new( Label::new( "Can't find Main component." ) ).erased() };
            match DefaultWidgetBuilder::build_widget( &params_stack ) {
                Ok(widget) => widget.erased(),
                Err(e) => NewWidget::new( Label::new( format!("{e:#?}") ) ).erased()
            }
        }
        Err( e ) => {
            let text = format!("{e:#?}\n{}", render_error(src, e.span.clone(),3));
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
        next_task: String::new(),
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