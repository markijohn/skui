use std::collections::{HashMap, HashSet};

use masonry::core::{NewWidget, Properties, Widget, WidgetOptions, WidgetTag};
use masonry::layout::{Length};
use masonry::properties::Padding;
use masonry::widgets::{Align, Button, Canvas, Checkbox, Flex, FlexParams, Grid, GridParams, Image, IndexedStack, Label, Passthrough, Portal, ProgressBar, Prose, ResizeObserver, SizedBox, Slider, Spinner, Split, TextArea, TextInput, VariableLabel};
use skui::{Component, Number, Parameters, SKUIParseError, TokensAndSpan, Value, SKUI};
use crate::params::{AlignArgs, ArgumentError, ButtonArgs, CheckboxArgs, FlexArgs, FlexItemArgs, FlexSpacerArgs, FromParams, GridArgs, GridParamsArgs, IndexedStackArgs, LabelArgs, ParamsStack, PassthroughArgs, PortalArgs, ProgressBarArgs, ProseArgs, ResizeObserverArgs, SizedBoxArgs, SliderArgs, SplitArgs, TextAreaArgs, TextInputArgs};


type Result<T> = std::result::Result<T, Error>;

#[derive(Debug,Clone)]
pub enum Error {
    RootComponentNotFound,

    UnknownComponent(String),
    RequiredChildren(usize),
    AtLeastOneRequired,
    ExactlyTwoChildRequired,
    ParseError(SKUIParseError),
    InvalidParameter(ArgumentError),
    GridChildMustBeItem,
    MultipleChildDefinitions(String)
}

impl From<SKUIParseError> for Error {
    fn from(e:SKUIParseError) -> Self {
        Error::ParseError(e)
    }
}

impl From<ArgumentError> for Error {
    fn from(e:ArgumentError) -> Self {
        Error::InvalidParameter(e)
    }
}

// static WID_TABLE: std::sync::LazyLock<std::sync::RwLock<HashMap<String, &'static str>>> =
//     std::sync::LazyLock::new(|| std::sync::RwLock::new(HashMap::new()) );
//
// fn get_widget_id(map_id: &str) -> &'static str {
//     if let Some(&id) = WID_TABLE.read().unwrap().get(map_id) {
//         return id;
//     }
//     let leaked: &'static str = Box::leak(map_id.to_string().into_boxed_str());
//     WID_TABLE.write().unwrap().insert(map_id.to_string(), leaked);
//     leaked
// }

static WID_TABLE: std::sync::LazyLock<std::sync::RwLock<HashMap<String, &'static str>>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(HashMap::new()) );

unsafe fn get_widget_id(map_id: &str) -> &'static str {
    if let Some(&id) = WID_TABLE.read().unwrap().get(map_id) {
        return id;
    }
    let leaked: &'static str = Box::leak(map_id.to_string().into_boxed_str());
    WID_TABLE.write().unwrap().insert(map_id.to_string(), leaked);
    leaked
}

pub unsafe fn get_widget_tag<W:Widget>(map_id: &str) -> WidgetTag<W> {
    unsafe { WidgetTag::<W>::named( get_widget_id(map_id) ) }
}

fn get_main_component<'a>(skui:&'a SKUI) -> Result<&'a Component<'a>> {
    let none_id_comp = skui.components.iter().find( |&c| c.id.is_none() );
    let main_id_comp = skui.components.iter().find( |&c| c.name == "Main" );
    main_id_comp.or(none_id_comp).ok_or_else(|| Error::RootComponentNotFound)
}

pub fn build_main_widget(src:&str) -> Result<NewWidget<impl Widget + ?Sized >> {
    let tks = TokensAndSpan::new( src );
    let skui = SKUI::parse( &tks )?;
    let main_comp = get_main_component( &skui )?;
    let widget = build_widget(&main_comp, &skui.components, None);
    widget
}

// fn wrap_new_widget<W: Widget>(comp:&Component, mut widget:W) -> NewWidget<impl Widget + ?Sized> {
//     let mut props = Properties::new();
//     if let Some(v) = comp.properties.get("padding").and_then(|v| v.as_f64()) { props = props.with(Padding::all(v)) }
//     let wid = comp.id.map(|id| { WidgetTag::<W>::named("text_input") });
//     let wopts = WidgetOptions::default();
//     NewWidget::new_with(widget, wid, wopts, props).erased()
// }

macro_rules! wrap_new {
    ($comp:ident, $widget:ident) => { {
        let mut props = Properties::new();
        if let Some(v) = $comp.properties.get("padding").and_then(|v| v.as_f64()) { props = props.with(Padding::all(v)); }
        //let wid = $comp.id.map( |id| { WidgetTag::named(id) } );
        let wid = $comp.id.map( |id| { unsafe { get_widget_tag(id) } } );
        let wopts = WidgetOptions::default();
        NewWidget::new_with($widget, wid, wopts, props).erased()
    } }
}

fn build_widget<'a>(comp:&'a Component, root_comps:&'a [Component],caller_params:Option<&'a Parameters>) -> Result<NewWidget<impl Widget + ?Sized + use<>>> {
    let params_stack = ParamsStack::new(caller_params, &comp.params);
    let v = match comp.name {
        "Align" => {
            let align_args = AlignArgs::from_params(&params_stack)?;
            let child = NewWidget::new(Label::new(""));
            let align = Align::new( align_args.unit_point, child);
            wrap_new!( comp, align )
        }
        "Button" => {
            let button_args = ButtonArgs::from_params(&params_stack)?;
            let btn = Button::new( NewWidget::new(Label::new(button_args.text)) );
            wrap_new!( comp, btn )
        }
        "Canvas" => {
            let canvas = Canvas::default();
            wrap_new!( comp, canvas )
        }
        "CheckBox" => {
            let checkbox_args = CheckboxArgs::from_params(&params_stack)?;
            let check_box = Checkbox::new( checkbox_args.checked.unwrap_or(false), checkbox_args.text );
            wrap_new!( comp, check_box )
        }
        "Flex" => {
            let flex_args = FlexArgs::from_params(&params_stack)?;
            let mut flex = Flex::for_axis(flex_args.axis);
            if let Some(main_axis_align) = flex_args.main_axis_alignment { flex = flex.main_axis_alignment(main_axis_align);}
            if let Some(cross_axis_align) = flex_args.cross_axis_alignment { println!("{cross_axis_align:#?}"); flex = flex.cross_axis_alignment(cross_axis_align);}
            for mut c in comp.children.iter() {
                let item_wrap = root_comps.iter()
                    .find(|rc|
                        rc.name == c.name
                            && rc.name != "Item"
                            && rc.name != "Spacing"
                            && rc.children.len() == 1
                            && rc.children[0].name == "Item").map( |rc| &rc.children[0]);
                if let Some(item_wrap) = item_wrap {
                    c = item_wrap;
                }

                match c.name {
                    "Item" => {
                        let params_stack = ParamsStack::new(None, &c.params);
                        let item_args = FlexItemArgs::from_params(&params_stack)?;
                        let item_comp = build_widget(item_args.comp, root_comps, None)?;
                        let params = FlexParams::new(item_args.flex, item_args.basis, item_args.alignment);
                        flex = flex.with( item_comp, params );
                    }
                    "Spacing" => {
                        let params_stack = ParamsStack::new(None, &c.params);
                        let spacer_args = FlexSpacerArgs::from_params(&params_stack)?;
                        flex = match spacer_args.value {
                            Number::I64(v) => flex.with_fixed_spacer( Length::const_px(v as _) ),
                            Number::F64(v) => flex.with_spacer(v)
                        }
                    }
                    _ => {
                        flex = flex.with_fixed( build_widget(c, root_comps, None)? );
                    }
                }
            }
            wrap_new!( comp, flex )
        }
        "Grid" => {
            let grid_args = GridArgs::from_params(&params_stack)?;
            let mut grid = Grid::with_dimensions( grid_args.x, grid_args.y );
            for mut c in comp.children.iter() {
                let item_wrap = root_comps.iter()
                    .find(|rc|
                        rc.name == c.name
                            && rc.name != "Item"
                            && rc.children.len() == 1
                            && rc.children[0].name == "Item").map( |rc| &rc.children[0]);
                if let Some(item_wrap) = item_wrap {
                    c = item_wrap;
                }
                
                match c.name {
                    "Item" => {
                        let params_stack = ParamsStack::new(None, &c.params);
                        let item_args = GridParamsArgs::from_params(&params_stack)?;
                        let item_comp = build_widget(item_args.comp, root_comps, None)?;
                        let params = GridParams::new(item_args.x, item_args.y, item_args.w.unwrap_or(1), item_args.h.unwrap_or(1));
                        grid = grid.with(item_comp, params);
                    }
                    _ => {
                        return Err(Error::GridChildMustBeItem)
                    }
                }
            }
            wrap_new!( comp, grid )
        }
        "Image" => {
            //let image = Image::new();
            // check image feature

            // check reqwest feature

            todo!()
        }
        "IndexedStack" => {
            let indexed_args = IndexedStackArgs::from_params(&params_stack)?;
            let mut indexed_stack = IndexedStack::new();
            for c in comp.children.iter() {
                match c.name {
                    "Item" => {
                        let comp = build_widget(c, root_comps, None)?;
                        indexed_stack = indexed_stack.with(comp);
                    }
                    _ => {
                        return Err(Error::GridChildMustBeItem)
                    }
                }
            }
            indexed_stack = indexed_stack.with_active_child(indexed_args.index);
            wrap_new!( comp, indexed_stack )
        }
        "Label" => {
            let label_param = LabelArgs::from_params(&params_stack)?;
            let label = Label::new(label_param.text);
            wrap_new!( comp, label )
        }
        "Passthrough" => {
            let passthrough_args = PassthroughArgs::from_params(&params_stack)?;
            let passthrough = Passthrough::new( build_widget(passthrough_args.comp, root_comps, None)? );
            wrap_new!( comp, passthrough )
        }
        "Portal" => {
            // let portal_args = PortalArgs::from_params(&params_stack)?;
            // let portal = Portal::new( build_widget(portal_args.comp, None)? );
            // NewWidget::new( Passthrough::new( portal ) ).erased()
            todo!()
        }
        "ProgressBar" => {
            let progress_bar_args = ProgressBarArgs::from_params(&params_stack)?;
            let progress_bar = ProgressBar::new( progress_bar_args.progress );
            wrap_new!( comp, progress_bar )
        }
        "Prose" => {
            let prose_args = ProseArgs::from_params(&params_stack)?;
            let mut prose = Prose::new(&prose_args.text);
            if let Some(flag) = prose_args.clip { prose = prose.with_clip(flag); }
            wrap_new!( comp, prose )
        }
        "ResizeObserver" => {
            //check one child
            let args = ResizeObserverArgs::from_params(&params_stack)?;
            let resize_observer = ResizeObserver::new( build_widget(args.comp, root_comps, None)? );
            wrap_new!( comp, resize_observer )
        }
        "SizedBox" => {
            let args = SizedBoxArgs::from_params(&params_stack)?;
            let mut sized_box = SizedBox::new( build_widget(args.comp, root_comps, None)? );
            if let Some(width) = args.width { sized_box = sized_box.width( Length::px( width ) ); }
            if let Some(height) = args.height { sized_box = sized_box.width( Length::px( height ) ); }
            wrap_new!( comp, sized_box )
        }
        "Slider" => {
            let args = SliderArgs::from_params(&params_stack)?;
            let mut slider = Slider::new(args.min, args.max, args.value);
            if let Some(step) = args.step { slider = slider.with_step(step); }
            wrap_new!( comp, slider )
        }
        "Spinner" => {
            let spinner = Spinner::new();
            wrap_new!( comp, spinner )
        }
        "Split" => {
            let args = SplitArgs::from_params(&params_stack)?;
            let (first,second) = if let (Some(first),Some(second)) = (args.first, args.second) {
                (first,second)
            } else if comp.children.len() == 2 {
                (&comp.children[0], &comp.children[1])
            } else {
                return Err(Error::ExactlyTwoChildRequired)
            };
            let split = Split::new(
                build_widget(&first, root_comps, None)?,
                build_widget(&second, root_comps, None)?
            );
            wrap_new!( comp, split )
        }
        "TextArea" => {
            let args = TextAreaArgs::from_params(&params_stack)?;
            if args.editable.unwrap_or(true) {
                let mut text_area = TextArea::<true>::new(args.text.unwrap_or(""));
                text_area = text_area.with_word_wrap( false );
                if let Some(align) = args.alignment { text_area = text_area.with_text_alignment(align); }
                if let Some(insert_newline) = args.insert_newline { text_area = text_area.with_insert_newline( insert_newline ); }
                if let Some(hint) = args.hint { text_area = text_area.with_hint(hint); }
                wrap_new!( comp, text_area )
            } else {
                let mut text_area = TextArea::<false>::new(args.text.unwrap_or(""));
                text_area = text_area.with_word_wrap( false );
                if let Some(align) = args.alignment { text_area = text_area.with_text_alignment(align); }
                if let Some(insert_newline) = args.insert_newline { text_area = text_area.with_insert_newline( insert_newline ); }
                if let Some(hint) = args.hint { text_area = text_area.with_hint(hint); }
                wrap_new!( comp, text_area )
            }
        }
        "TextInput" => {
            let args = TextInputArgs::from_params(&params_stack)?;
            let mut text_input = TextInput::new(args.text.unwrap_or(""));
            if let Some(placeholder) = args.placeholder { text_input = text_input.with_placeholder(placeholder); }
            if let Some(clip) = args.clip { text_input = text_input.with_clip(clip); }
            if let Some(alignment) = args.alignment { text_input = text_input.with_text_alignment(alignment); };
            wrap_new!( comp, text_input )
        }
        "VariableLabel" => {
            let text = "";
            let var_label = VariableLabel::new(text);
            wrap_new!( comp, var_label )
        }
        name @ _ => {
            //check custom components
            let is_root = caller_params.is_none();
            if is_root {
                let caller_param = Some(&comp.params);
                if let Some(other_comp) = root_comps.iter().find(|comp| comp.name == name) {
                    build_widget(other_comp, root_comps, caller_param)?
                } else {
                    return Err(Error::UnknownComponent(name.to_string()))
                }

            } else {
                if comp.children.len() != 1 {
                    return Err(Error::MultipleChildDefinitions(name.to_string()))
                }
                build_widget(&comp.children[0], root_comps, None)?
            }
        },
    };
    Ok(v)
}
