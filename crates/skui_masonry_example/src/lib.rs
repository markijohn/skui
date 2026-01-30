//mod builder;
pub mod params;
mod q;
mod style;

use std::collections::HashMap;
use std::marker::PhantomData;
use masonry::core::{BrushIndex, ErasedAction, NewWidget, Properties, Widget, WidgetOptions, WidgetTag};
use masonry::layout::Length;
use masonry::peniko::color::AlphaColor;
use masonry::properties::{Background, Gap, Padding};
use masonry::widgets::{Align, Button, Canvas, Checkbox, Flex, FlexParams, Grid, GridParams, Image, IndexedStack, Label, Passthrough, Portal, ProgressBar, Prose, ResizeObserver, SizedBox, Slider, Spinner, Split, TextArea, TextInput, VariableLabel};
use skui::{Component, CssValue, Number, Parameters, SKUIParseError, TokenAndSpan, SKUI};
use crate::params::{AlignArgs, ArgumentError, ButtonArgs, CheckboxArgs, FlexArgs, FlexItemArgs, FlexSpacerArgs, FromParams, GridArgs, GridParamsArgs, IndexedStackArgs, LabelArgs, ParamsStack, PassthroughArgs, PortalArgs, ProgressBarArgs, ProseArgs, ResizeObserverArgs, SizedBoxArgs, SliderArgs, SplitArgs, TextAreaArgs, TextInputArgs, VariableLabelArgs};
use std::str::FromStr;
use masonry::parley::{Brush, FontWeight, StyleProperty};

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

static WID_TABLE: std::sync::LazyLock<std::sync::RwLock<HashMap<String, &'static str>>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(HashMap::new()) );




macro_rules! impl_default_widget_builder {
    ( $name:ident { $($comp:ident),* } ) => {
        impl <P:CustomPropertyBuilder> RootWidgetBuilder for $name <P> {
            fn build_widget<'a>(params_stack:&ParamsStack<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error> {
                match params_stack.component.name {
                    $(
                    $comp::WIDGET_NAME => $comp::build::<Self>(params_stack).map(|v| v.erased()) ,
                    )*
                    _ => Err( Error::UnknownComponent( format!("{} -> {}", params_stack.fn_name, params_stack.component.name) ) )
                }
            }

            fn build_custom_properties<'a>(props: &mut Properties, c: &Component<'a>, skui: &SKUI<'a>) {
                P::build_properties(props, c, skui);
            }
        }
    }
}

pub trait CustomPropertyBuilder {
    fn build_properties<'a>(props:&mut Properties, c:&Component<'a>, skui:&SKUI<'a>);
}

pub struct EmptyPropertyBuilder;
impl CustomPropertyBuilder for EmptyPropertyBuilder {
    fn build_properties<'a>(props: &mut Properties, c: &Component<'a>, skui: &SKUI<'a>) {
        //None
    }
}


pub struct DefaultWidgetBuilder<P> {
    p : PhantomData<P>
}

pub type BasicWidgetBuilder = DefaultWidgetBuilder<EmptyPropertyBuilder>;


impl_default_widget_builder!(DefaultWidgetBuilder {Align,Button,Canvas,Checkbox,Flex,Grid,Image,
            IndexedStack,Label,Passthrough,Portal,ProgressBar,Prose,ResizeObserver,
            SizedBox,Slider,Spinner,Split,TextAreaEditable,TextInput,VariableLabel});



pub trait RootWidgetBuilder {
    unsafe fn get_widget_id(map_id: &str) -> &'static str {
        if let Some(&id) = WID_TABLE.read().unwrap().get(map_id) {
            return id;
        }
        let leaked: &'static str = Box::leak(map_id.to_string().into_boxed_str());
        WID_TABLE.write().unwrap().insert(map_id.to_string(), leaked);
        leaked
    }

    unsafe fn get_widget_tag<W:Widget>(map_id: &str) -> WidgetTag<W> {
        unsafe { WidgetTag::<W>::named( Self::get_widget_id(map_id) ) }
    }

    fn build_widget<'a>(params_stack:&ParamsStack<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error>;

    fn build_styles<'a>(build_prop:bool, build_styles:bool, c:&Component<'a>, skui:&SKUI<'a>) -> (Properties,Vec<StyleProperty<'static,BrushIndex>>) {
        let mut props = Properties::new();
        let mut styles = vec![];
        let mut parents = vec![];
        let Some(main) = skui.get_main_component() else { return (props, styles) };
        main.component.find( &mut parents, c );
        skui.get_styles(parents.as_slice(), c)
            .for_each( |style| {
                style::style_parse(build_prop, build_styles, style, &mut props, &mut styles);
            });
        Self::build_custom_properties(&mut props, c, skui);
        (props, styles)
    }

    fn build_custom_properties<'a>(props: &mut Properties, c: &Component<'a>, skui: &SKUI<'a>);
}

type MasonryStyle = StyleProperty<'static,BrushIndex>;
type MasonryStyles = Vec<StyleProperty<'static,BrushIndex>>;

pub trait WidgetBuilder {
    const WIDGET_NAME: &'static str;
    const BUILD_PROPERTIES:bool = true;
    const BUILD_STYLES:bool = false;
    type TargetWidget: Widget;

    fn build<'a,B:RootWidgetBuilder>(params_stack:&ParamsStack<'a>)  -> Result<NewWidget<impl Widget + ?Sized>, Error> {
        let (props, styles) = B::build_styles(Self::BUILD_PROPERTIES, Self::BUILD_STYLES, &params_stack.component, &params_stack.skui) ;
        let mut widget = <Self as WidgetBuilder>::build_target::<B>(params_stack)?;
        if Self::BUILD_STYLES {
            for s in styles.into_iter() {
                widget = Self::apply_style::<B>( widget, s);
            }
        }
        let wid = params_stack.get_id().map( |id| { unsafe { B::get_widget_tag(id) } } );
        let wopts = WidgetOptions::default();

        //let props = B::build_properties(&params_stack.component, &params_stack.skui);

        Ok( NewWidget::new_with(widget, wid, wopts, props).erased() )
    }

    fn build_target<'a,B:RootWidgetBuilder>(params_stack:&ParamsStack<'a>) -> Result<Self::TargetWidget, Error>;

    fn apply_style<'a,B:RootWidgetBuilder>(target:Self::TargetWidget, style:MasonryStyle) -> Self::TargetWidget {
        target
    }
}

impl WidgetBuilder for Align {
    const WIDGET_NAME: &'static str = "Align";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let align_args = AlignArgs::from_params(params_stack)?;
        let child = B::build_widget( &params_stack.new_stack(align_args.comp) )?;
        let widget = Align::new( align_args.unit_point, child );
        Ok( widget )
    }
}

impl WidgetBuilder for Button {
    const WIDGET_NAME: &'static str = "Button";
    type TargetWidget = Self;
    const BUILD_STYLES:bool = true;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        //let button_args = ButtonArgs::from_params(params_stack)?;
        let widget = Button::new( Label::build::<B>(params_stack)? );
        Ok( widget )
    }
}

impl WidgetBuilder for Canvas {
    const WIDGET_NAME: &'static str = "Canvas";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let widget = Canvas::default();
        Ok( widget )
    }
}

impl WidgetBuilder for Checkbox {
    const WIDGET_NAME: &'static str = "Checkbox";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let checkbox_args = CheckboxArgs::from_params(params_stack)?;
        let widget = Checkbox::new( checkbox_args.checked.unwrap_or(false), checkbox_args.text );
        Ok( widget )
    }
}

impl WidgetBuilder for Flex {
    const WIDGET_NAME: &'static str = "Flex";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        println!("\n{:#?}", params_stack);
        let flex_args = FlexArgs::from_params(params_stack)?;
        let mut widget = Flex::for_axis(flex_args.axis);
        if let Some(main_axis_align) = flex_args.main_axis_alignment { widget = widget.main_axis_alignment(main_axis_align);}
        if let Some(cross_axis_align) = flex_args.cross_axis_alignment { widget = widget.cross_axis_alignment(cross_axis_align);}
        for mut c in params_stack.children() {
            let flex_child_stack = params_stack.new_stack( c );
            match flex_child_stack.component.name {
                "FlexItem" => {
                    let item_args = FlexItemArgs::from_params( &flex_child_stack )?;
                    let item_comp = B::build_widget(&flex_child_stack.new_stack(item_args.comp))?;
                    let params = FlexParams::new(item_args.flex, item_args.basis, item_args.alignment);
                    widget = widget.with( item_comp, params );
                }
                "FlexSpace" => {
                    let inner_stack = params_stack.new_stack(c);
                    let spacer_args = FlexSpacerArgs::from_params(&inner_stack)?;
                    widget = match spacer_args.value {
                        Number::I64(v) => widget.with_fixed_spacer( Length::const_px(v as _) ),
                        Number::F64(v) => widget.with_spacer(v)
                    }
                }
                _ => {
                    let child = B::build_widget(&flex_child_stack)?;
                    widget = widget.with_fixed( child );
                }
            }
        }
        Ok( widget )
    }
}

impl WidgetBuilder for Grid {
    const WIDGET_NAME: &'static str = "Grid";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let grid_args = GridArgs::from_params(params_stack)?;
        let mut widget = Grid::with_dimensions( grid_args.x, grid_args.y );

        for c in params_stack.children() {
            let grid_child_stack = params_stack.new_stack(c);
            match grid_child_stack.component.name {
                "GridItem" => {
                    let item_args = GridParamsArgs::from_params(&grid_child_stack)?;
                    let item_comp = B::build_widget(&grid_child_stack.new_stack(item_args.comp))?;
                    let params = GridParams::new(item_args.x, item_args.y, item_args.w.unwrap_or(1), item_args.h.unwrap_or(1));
                    widget = widget.with(item_comp, params);
                }
                _ => {
                    return Err(Error::GridChildMustBeItem)
                }
            }
        }
        Ok( widget )
    }
}

impl WidgetBuilder for Image {
    const WIDGET_NAME: &'static str = "Image";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        todo!()
    }
}

impl WidgetBuilder for IndexedStack {
    const WIDGET_NAME: &'static str = "IndexedStack";
    type TargetWidget = Self;
    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let indexed_args = IndexedStackArgs::from_params(params_stack)?;
        let mut widget = IndexedStack::new();

        for c in params_stack.children() {
            match c.name {
                "Item" => {
                    let comp = B::build_widget( &params_stack.new_stack(c) )?;
                    widget = widget.with(comp);
                }
                _ => {
                    return Err(Error::GridChildMustBeItem)
                }
            }
        }
        widget = widget.with_active_child(indexed_args.index);
        Ok( widget )
    }
}

impl WidgetBuilder for Label {
    const WIDGET_NAME: &'static str = "Label";
    type TargetWidget = Self;
    const BUILD_STYLES:bool = true;
    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let label_args = LabelArgs::from_params(params_stack)?;
        let widget = Label::new(label_args.text);
        Ok( widget )
    }

    fn apply_style<'a, B: RootWidgetBuilder>(target: Self::TargetWidget, style: MasonryStyle) -> Self::TargetWidget {
        target.with_style(style)
    }
}

impl WidgetBuilder for Passthrough {
    const WIDGET_NAME: &'static str = "Passthrough";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let passthrough_args = PassthroughArgs::from_params(params_stack)?;
        let widget = Passthrough::new( B::build_widget( &params_stack.new_stack(passthrough_args.comp) )? );
        Ok( widget )
    }
}

impl WidgetBuilder for Portal<Label> {
    const WIDGET_NAME: &'static str = "Portal";
    type TargetWidget = Label; //dont care

    fn build<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error> {
        let portal_args = PortalArgs::from_params(&params_stack)?;
        let widget = Portal::new( B::build_widget( &params_stack.new_stack(portal_args.comp) )?.erased() );
        let wid = params_stack.get_id().map( |id| { unsafe { B::get_widget_tag(id) } } );
        let wopts = WidgetOptions::default();
        let props = Properties::new();
        Ok( NewWidget::new_with(widget, wid, wopts, props).erased() )
    }

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        unreachable!()
    }
}

impl WidgetBuilder for ProgressBar {
    const WIDGET_NAME: &'static str = "ProgressBar";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let progress_bar_args = ProgressBarArgs::from_params(params_stack)?;
        let widget = ProgressBar::new( progress_bar_args.progress );
        Ok( widget )
    }
}

impl WidgetBuilder for Prose {
    const WIDGET_NAME: &'static str = "Prose";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let prose_args = ProseArgs::from_params(params_stack)?;
        let mut widget = Prose::new(&prose_args.text);
        if let Some(flag) = prose_args.clip { widget = widget.with_clip(flag); }
        Ok( widget )
    }
}

impl WidgetBuilder for ResizeObserver {
    const WIDGET_NAME: &'static str = "ResizeObserver";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let args = ResizeObserverArgs::from_params(params_stack)?;
        let widget = ResizeObserver::new( B::build_widget( &params_stack.new_stack(args.comp) )? );
        Ok( widget )
    }
}

impl WidgetBuilder for SizedBox {
    const WIDGET_NAME: &'static str = "SizedBox";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let args = SizedBoxArgs::from_params(params_stack)?;
        let mut widget = SizedBox::new( B::build_widget( &params_stack.new_stack(args.comp) )? );
        if let Some(width) = args.width { widget = widget.width( Length::px( width ) ); }
        if let Some(height) = args.height { widget = widget.width( Length::px( height ) ); }
        Ok( widget )
    }
}

impl WidgetBuilder for Slider {
    const WIDGET_NAME: &'static str = "Slider";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let args = SliderArgs::from_params(&params_stack)?;
        let mut widget = Slider::new(args.min, args.max, args.value);
        if let Some(step) = args.step { widget = widget.with_step(step); }
        Ok( widget )
    }
}

impl WidgetBuilder for Spinner {
    const WIDGET_NAME: &'static str = "Spinner";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let widget = Spinner::default();
        Ok( widget )
    }
}

impl WidgetBuilder for Split<dyn Widget<Action=ErasedAction>,dyn Widget<Action=ErasedAction>> {
    const WIDGET_NAME: &'static str = "Split";
    type TargetWidget = Label;

    fn build<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error> {
        let args = SplitArgs::from_params(params_stack)?;
        let (first,second) = if let (Some(first),Some(second)) = (args.first, args.second) {
            (first,second)
        } else if params_stack.children().count() == 2 {
            let mut iter = params_stack.children();
            (iter.next().unwrap(), iter.next().unwrap() )
        } else {
            return Err(Error::ExactlyTwoChildRequired)
        };
        let widget = Split::new(
            B::build_widget( &params_stack.new_stack(first) )?.erased(),
            B::build_widget( &params_stack.new_stack(second) )?.erased()
        );
        let wid = params_stack.get_id().map( |id| { unsafe { B::get_widget_tag(id) } } );
        let wopts = WidgetOptions::default();
        let (props, _styles) = B::build_styles(true,false,&params_stack.component,&params_stack.skui);
        Ok( NewWidget::new_with(widget, wid, wopts, props).erased() )
    }

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        unreachable!()
    }
}

pub type TextAreaEditable = TextArea<true>;
impl <const USER_EDITABLE:bool> WidgetBuilder for TextArea<USER_EDITABLE> {
    const WIDGET_NAME: &'static str = "TextArea";
    type TargetWidget = Label;

    fn build<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error> {
        let args = TextAreaArgs::from_params(params_stack)?;
        let (props,styles) = B::build_styles(true,true,&params_stack.component, &params_stack.skui);
        if args.editable.unwrap_or(true) {
            let mut widget = TextArea::<true>::new(args.text.unwrap_or(""));
            let wid = params_stack.get_id().map( |id| { unsafe { B::get_widget_tag(id) } } );
            let wopts = WidgetOptions::default();
            for s in styles.into_iter() {
                widget = widget.with_style(s);
            }
            Ok( NewWidget::new_with(widget, wid, wopts, props).erased() )
        } else {
            let mut widget = TextArea::<false>::new(args.text.unwrap_or(""));
            let wid = params_stack.get_id().map( |id| { unsafe { B::get_widget_tag(id) } } );
            let wopts = WidgetOptions::default();
            for s in styles.into_iter() {
                widget = widget.with_style(s);
            }
            Ok( NewWidget::new_with(widget, wid, wopts, props).erased() )
        }
    }

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        unreachable!()
    }
}

impl WidgetBuilder for TextInput {
    const WIDGET_NAME: &'static str = "TextInput";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let args = TextInputArgs::from_params(params_stack)?;
        let mut widget = TextInput::new(args.text.unwrap_or(""));
        if let Some(placeholder) = args.placeholder { widget = widget.with_placeholder(placeholder); }
        if let Some(clip) = args.clip { widget = widget.with_clip(clip); }
        if let Some(alignment) = args.alignment { widget = widget.with_text_alignment(alignment); };
        Ok( widget )
    }
}

impl WidgetBuilder for VariableLabel {
    const WIDGET_NAME: &'static str = "VariableLabel";
    type TargetWidget = Self;

    fn build_target<'a, B: RootWidgetBuilder>(params_stack: &ParamsStack<'a>) -> Result<Self::TargetWidget, Error> {
        let args = VariableLabelArgs::from_params(params_stack)?;
        let mut widget = VariableLabel::new(args.text);
        widget = widget.with_initial_weight( args.weight.unwrap_or(FontWeight::NORMAL.value()) );
        Ok( widget )
    }
}
