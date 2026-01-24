use masonry::core::{NewWidget, Widget};
use masonry::layout::{Length};
use masonry::widgets::{Align, Button, Canvas, Checkbox, Flex, FlexParams, Grid, GridParams, Image, IndexedStack, Label, Passthrough, Portal, ProgressBar, Prose, ResizeObserver, SizedBox, Slider, Spinner, Split, TextArea, TextInput, VariableLabel};
use skui::{Component, Number, Parameters, SKUIParseError, SKUI};
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

fn get_main_component(skui:&SKUI) -> Result<&Component> {
    let none_id_comp = skui.components.iter().find( |&c| c.id.is_none() );
    let main_id_comp = skui.components.iter().find( |&c| c.id.as_ref().map(String::as_str).unwrap_or("") == "main" );
    none_id_comp.or(main_id_comp).ok_or_else(|| Error::RootComponentNotFound)
}

pub fn build_root_widget(src:&str) -> Result<NewWidget<impl Widget + ?Sized >> {
    let skui = SKUI::parse( src )?;
    let main_comp = get_main_component( &skui )?;
    let widget = build_widget(&main_comp, None);
    skui.components.len();
    widget
}

fn build_widget(comp:&Component, caller_params:Option<&Parameters>) -> Result<NewWidget<impl Widget + ?Sized + use<>>> {
    let params_stack = ParamsStack::new(caller_params, &comp.params);
    let v = match comp.name.as_str() {
        "Align" => {
            let align_args = AlignArgs::from_params(&params_stack)?;
            let child = NewWidget::new(Label::new(""));
            let align = Align::new( align_args.unit_point, child);
            NewWidget::new(align).erased()
        }
        "Button" => {
            let button_args = ButtonArgs::from_params(&params_stack)?;
            let btn = Button::new( NewWidget::new(Label::new(button_args.text)) );
            NewWidget::new(btn).erased()
        }
        "Canvas" => {
            let canvas = Canvas::default();
            NewWidget::new(canvas).erased()
        }
        "CheckBox" => {
            let checkbox_args = CheckboxArgs::from_params(&params_stack)?;
            let check_box = Checkbox::new( checkbox_args.checked.unwrap_or(false), checkbox_args.text );
            NewWidget::new(check_box).erased()
        }
        "Flex" => {
            let flex_args = FlexArgs::from_params(&params_stack)?;
            let mut flex = Flex::for_axis(flex_args.axis);
            if let Some(main_axis_align) = flex_args.main_axis_alignment { flex = flex.main_axis_alignment(main_axis_align);}
            if let Some(cross_axis_align) = flex_args.cross_axis_alignment { flex = flex.cross_axis_alignment(cross_axis_align);}
            for c in comp.children.iter() {
                match c.name.as_str() {
                    "Item" => {
                        let item_args = FlexItemArgs::from_params(&params_stack)?;
                        let item_comp = build_widget(item_args.comp, None)?;
                        let params = FlexParams::new(item_args.flex, item_args.basis, item_args.alignment);
                        flex = flex.with( item_comp, params );
                    }
                    "Space" => {
                        let spacer_args = FlexSpacerArgs::from_params(&params_stack)?;
                        flex = match spacer_args.value {
                            Number::I64(v) => flex.with_fixed_spacer( Length::const_px(v as _) ),
                            Number::F64(v) => flex.with_spacer(v)
                        }
                    }
                    _ => {
                        flex = flex.with_fixed( build_widget(c, None)? );
                    }
                }
            }
            NewWidget::new( flex ).erased()
        }
        "Grid" => {
            let grid_args = GridArgs::from_params(&params_stack)?;
            let mut grid = Grid::with_dimensions( grid_args.x, grid_args.y );
            for c in comp.children.iter() {
                match c.name.as_str() {
                    "Item" => {
                        let item_args = GridParamsArgs::from_params(&params_stack)?;
                        let item_comp = build_widget(item_args.comp, None)?;
                        let params = GridParams::new(item_args.x, item_args.y, item_args.w.unwrap_or(1), item_args.h.unwrap_or(1));
                        grid = grid.with(item_comp, params);
                    }
                    _ => {
                        return Err(Error::GridChildMustBeItem)
                    }
                }
            }
            NewWidget::new(grid).erased()
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
                match c.name.as_str() {
                    "Item" => {
                        let comp = build_widget(c, None)?;
                        indexed_stack = indexed_stack.with(comp);
                    }
                    _ => {
                        return Err(Error::GridChildMustBeItem)
                    }
                }
            }
            indexed_stack = indexed_stack.with_active_child(indexed_args.index);
            NewWidget::new(indexed_stack).erased()
        }
        "Label" => {
            let label_param = LabelArgs::from_params(&params_stack)?;
            let label = Label::new(label_param.text);
            NewWidget::new(label).erased()
        }
        "Passthrough" => {
            let passthrough_args = PassthroughArgs::from_params(&params_stack)?;
            let passthrough = Passthrough::new( build_widget(passthrough_args.comp, None)? );
            NewWidget::new(passthrough).erased()
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
            NewWidget::new(progress_bar).erased()
        }
        "Prose" => {
            let prose_args = ProseArgs::from_params(&params_stack)?;
            let mut prose = Prose::new(&prose_args.text);
            if let Some(flag) = prose_args.clip { prose = prose.with_clip(flag); }
            NewWidget::new(prose).erased()
        }
        "ResizeObserver" => {
            //check one child
            let args = ResizeObserverArgs::from_params(&params_stack)?;
            let resize_observer = ResizeObserver::new( build_widget(args.comp, None)? );
            NewWidget::new(resize_observer).erased()
        }
        "SizedBox" => {
            let args = SizedBoxArgs::from_params(&params_stack)?;
            let mut sized_box = SizedBox::new( build_widget(args.comp, None)? );
            if let Some(width) = args.width { sized_box = sized_box.width( Length::px( width ) ); }
            if let Some(height) = args.height { sized_box = sized_box.width( Length::px( height ) ); }
            NewWidget::new(sized_box).erased()
        }
        "Slider" => {
            let args = SliderArgs::from_params(&params_stack)?;
            let mut slider = Slider::new(args.min, args.max, args.value);
            if let Some(step) = args.step { slider = slider.with_step(step); }
            NewWidget::new(slider).erased()
        }
        "Spinner" => {
            NewWidget::new(Spinner::new( )).erased()
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
                build_widget(&first, None)?,
                build_widget(&second, None)?
            );
            NewWidget::new(split).erased()
        }
        "TextArea" => {
            let args = TextAreaArgs::from_params(&params_stack)?;
            if args.editable.unwrap_or(true) {
                let mut text_area = TextArea::<true>::new(args.text.unwrap_or(""));
                text_area = text_area.with_word_wrap( false );
                if let Some(align) = args.alignment { text_area = text_area.with_text_alignment(align); }
                if let Some(insert_newline) = args.insert_newline { text_area = text_area.with_insert_newline( insert_newline ); }
                if let Some(hint) = args.hint { text_area = text_area.with_hint(hint); }
                NewWidget::new(text_area).erased()
            } else {
                let mut text_area = TextArea::<false>::new(args.text.unwrap_or(""));
                text_area = text_area.with_word_wrap( false );
                if let Some(align) = args.alignment { text_area = text_area.with_text_alignment(align); }
                if let Some(insert_newline) = args.insert_newline { text_area = text_area.with_insert_newline( insert_newline ); }
                if let Some(hint) = args.hint { text_area = text_area.with_hint(hint); }
                NewWidget::new(text_area).erased()
            }
        }
        "TextInput" => {
            let args = TextInputArgs::from_params(&params_stack)?;
            let mut text_input = TextInput::new(args.text.unwrap_or(""));
            if let Some(placeholder) = args.placeholder { text_input = text_input.with_placeholder(placeholder); }
            if let Some(clip) = args.clip { text_input = text_input.with_clip(clip); }
            if let Some(alignment) = args.alignment { text_input = text_input.with_text_alignment(alignment); };
            NewWidget::new(text_input).erased()
        }
        "VariableLabel" => {
            let text = "";
            let var_label = VariableLabel::new(text);
            NewWidget::new(var_label).erased()
        }
        name @ _ => {
            //check custom components
            let caller_param = &comp.params;

            return Err(Error::UnknownComponent(name.to_string()))
        },
    };
    Ok(v)
}
