mod builder;
mod params;

use masonry::core::{ErasedAction, NewWidget, Properties, Widget, WidgetOptions};
use masonry::layout::Length;
use masonry::widgets::{Align, Button, Canvas, Checkbox, Flex, FlexParams, Label, Portal};
pub use builder::{Error, build_main_widget, get_widget_tag};
use skui::{Component, Number, Parameters, SKUI};
use crate::params::{AlignArgs, ButtonArgs, CheckboxArgs, FlexArgs, FlexItemArgs, FlexSpacerArgs, FromParams, ParamsStack};

macro_rules! impl_default_widget_builder {
    ( $name:ident { $($comp:ident),* } ) => {
        struct $name;

        impl MainWidgetBuilder for $name {
            fn name() -> &'static str { stringify!($name) }
            fn build_widget<'a>(caller_params:Option<&'a Parameters<'a>>, comp:&Component<'a>, skui:&'a SKUI<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error> {
                //let styles = styles.from(comp);
                match comp.name {
                    $(
                    $comp::WIDGET_NAME => $comp::build_widget::<Self>(caller_params, comp, skui).map(|v| v.erased()) ,
                    )*
                    _ => Err( Error::UnknownComponent( comp.name.to_string() ) )
                }
            }
        }
    }
}

#[macro_export]
macro_rules! impl_widget_builder {
    () => {
        impl_default_widget_builder!(
            Main {
            Align,Button,Canvas,CheckBox,Flex,Grid,Image,
            IndexedStack,Label,Passthrough,ProgressBar,Prose,ResizeObserver,
            SizedBox,Slider,Spinner,Split,TextArea,TextInput,VariableLabel }
        )
    }
}

impl_default_widget_builder!(Main {Align,Button,Canvas });



pub trait PropertiesExt {
    fn from_skui(comp:&Component, skui:&SKUI) -> Self;
}

impl PropertiesExt for Properties {
    fn from_skui(comp: &Component, skui: &SKUI) -> Self {
        let props = Properties::new();
        props
    }
}

pub trait MainWidgetBuilder {
    fn name() -> &'static str;
    fn build_widget<'a>(caller_params:Option<&'a Parameters<'a>>, comp:&Component<'a>, skui:&'a SKUI<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error>;
}

pub trait WidgetBuilder {
    const WIDGET_NAME: &'static str;
    fn build_widget<'a,B:MainWidgetBuilder>(caller_params:Option<&'a Parameters<'a>>, comp:&Component<'a>, skui:&'a SKUI<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error>;

}

impl WidgetBuilder for Align {
    const WIDGET_NAME: &'static str = "Align";
    fn build_widget<'a, B: MainWidgetBuilder>(caller_params: Option<&'a Parameters<'a>>, comp: &Component<'a>, skui: &'a SKUI<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error> {
        let params_stack = ParamsStack::new( caller_params, &comp.params );
        let align_args = AlignArgs::from_params(&params_stack)?;
        let child = B::build_widget( None, align_args.comp, skui )?;
        let align = Align::new( align_args.unit_point, child );
        Ok( NewWidget::new(align) )
    }
}

//WidgetData { param_stack, id, properties } parent_info, component, skui
//fn build_widget(caller:CallInfo, )

impl WidgetBuilder for Button {
    const WIDGET_NAME: &'static str = "Button";

    fn build_widget<'a, B: MainWidgetBuilder>(caller_params: Option<&'a Parameters<'a>>, comp: &Component<'a>, skui: &'a SKUI<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error> {
        let params_stack = ParamsStack::new( caller_params, &comp.params );
        let button_args = ButtonArgs::from_params(&params_stack)?;
        let widget = Button::new( NewWidget::new(Label::new(button_args.text)) );
        Ok( NewWidget::new(widget) )
    }
}

impl WidgetBuilder for Canvas {
    const WIDGET_NAME: &'static str = "Canvas";

    fn build_widget<'a, B: MainWidgetBuilder>(caller_params: Option<&'a Parameters<'a>>, comp: &Component<'a>, skui: &'a SKUI<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error> {
        let widget = Canvas::default();
        Ok( NewWidget::new(widget) )
    }
}

impl WidgetBuilder for Checkbox {
    const WIDGET_NAME: &'static str = "Checkbox";
    fn build_widget<'a, B: MainWidgetBuilder>(caller_params: Option<&'a Parameters<'a>>, comp: &Component<'a>, skui: &'a SKUI<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error> {
        let params_stack = ParamsStack::new( caller_params, &comp.params );
        let checkbox_args = CheckboxArgs::from_params(&params_stack)?;
        let check_box = Checkbox::new( checkbox_args.checked.unwrap_or(false), checkbox_args.text );
        Ok( NewWidget::new(check_box) )
    }
}

impl WidgetBuilder for Flex {
    const WIDGET_NAME: &'static str = "Flex";

    fn build_widget<'a, B: MainWidgetBuilder>(caller_params:Option<&'a Parameters<'a>>, comp: &Component<'a>, skui: &'a SKUI<'a>) -> Result<NewWidget<impl Widget + ?Sized>, Error> {
        let params_stack = ParamsStack::new( caller_params, &comp.params );
        let flex_args = FlexArgs::from_params(&params_stack)?;
        let mut widget = Flex::for_axis(flex_args.axis);
        if let Some(main_axis_align) = flex_args.main_axis_alignment { widget = widget.main_axis_alignment(main_axis_align);}
        if let Some(cross_axis_align) = flex_args.cross_axis_alignment { println!("{cross_axis_align:#?}"); widget = widget.cross_axis_alignment(cross_axis_align);}
        for mut c in comp.children.iter() {
            c = skui.get_lookup_scoped_component(c, &["FlexItem"]);

            match c.name {
                "FlexItem" => {
                    let params_stack = ParamsStack::new(None, &c.params);
                    let item_args = FlexItemArgs::from_params(&params_stack)?;
                    let item_comp = B::build_widget(None, item_args.comp, skui)?;
                    let params = FlexParams::new(item_args.flex, item_args.basis, item_args.alignment);
                    widget = widget.with( item_comp, params );
                }
                "Spacing" => {
                    let params_stack = ParamsStack::new(None, &c.params);
                    let spacer_args = FlexSpacerArgs::from_params(&params_stack)?;
                    widget = match spacer_args.value {
                        Number::I64(v) => widget.with_fixed_spacer( Length::const_px(v as _) ),
                        Number::F64(v) => widget.with_spacer(v)
                    }
                }
                _ => {
                    let child = B::build_widget(None, c, skui)?;
                    widget = widget.with_fixed( child );
                }
            }
        }

        if let Some("scroll") = comp.properties.get("overflow").and_then(|v|v.as_str()) {
            let flex = Portal::new( NewWidget::new(widget)  );
            Ok( NewWidget::new(flex).erased() )
        } else {
            Ok( NewWidget::new(widget).erased() )
        }
    }
}