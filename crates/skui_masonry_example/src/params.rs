use masonry::kurbo::Axis;
use masonry::layout::UnitPoint;
use masonry::properties::types::{CrossAxisAlignment, MainAxisAlignment};
use skui::{Parameters, Value};

/*
"Align" => {
let unit_point = UnitPoint::CENTER;
let child = NewWidget::new(Label::new(""));
let align = Align::new( unit_point, child);
NewWidget::new(align).erased()
}
"Button" => {
//let label_text:&str = comp.get_params( 0.or("text") ).unwrap_or("Unnamed");
let label_text = "unnamed";
let btn = Button::new( NewWidget::new(Label::new(label_text)) );
NewWidget::new(btn).erased()
}
"Canvas" => {
todo!()
}
"CheckBox" => {
//let label_text:&str = comp.get_bool( 1.or("checked") ).unwrap_or( false );
let default_checked = false;

//let label_text:&str = comp.get_params( 0.or("text") ).unwrap_or("Unnamed");
let label_text = "unnamed";
let check_box = Checkbox::new( default_checked, label_text );
NewWidget::new(check_box).erased()
}
"Flex" => {
let mut flex = Flex::row();

//Consume children
for c in comp.children.iter() {
match c.name.as_str() {
"item" => {
//let (weight, comp) = comp.get();
let weight = 0.;
let item_comp = Label::new("ITEM");
flex = flex.with( NewWidget::new(item_comp).erased(), weight );
}
other @ _ => {
flex = flex.with_fixed( build_widget_recurr(c, None)? );
}
}
}

NewWidget::new(flex).erased()
}
"CheckBox" => {
//let label_text:&str = comp.get_bool( 1.or("checked") ).unwrap_or( false );
let default_checked = false;

//let label_text:&str = comp.get_params( 0.or("text") ).unwrap_or("Unnamed");
let label_text = "unnamed";
let check_box = Checkbox::new( default_checked, label_text );
NewWidget::new(check_box).erased()
}
"Grid" => {
//let (x,y) = comp.get( 0.and(1) );
let (x,y) = (2,2);
let grid = Grid::with_dimensions(x,y);

//Consume children
NewWidget::new(grid).erased()
}
"Image" => {
// check image feature

// check reqwest feature

todo!()
}
"IndexedStack" => {
//let default_active_index = comp.get( 0 ).unwrap_or(0);
let default_active_index = 0;


let mut indexed_stack = IndexedStack::new();
indexed_stack = indexed_stack.with_active_child(default_active_index as usize);

//add children

NewWidget::new(indexed_stack).erased()
}
"Label" => {
//let label_text:&str = comp.get_bool( 1.or("checked") ).unwrap_or( false );
let label_text = "unnamed";
let label = Label::new(label_text);
NewWidget::new(label).erased()
}
"Passthrough" => {
//check one child
let passthrough = Passthrough::new( NewWidget::new(Label::new("")) );
NewWidget::new(passthrough).erased()
}
"Portal" => {
let single_child = NewWidget::new(Label::new(""));
let portal = Portal::new( single_child );
NewWidget::new(portal).erased()
}
"ProgressBar" => {
let progress = None;
let progress_bar = ProgressBar::new( progress );
NewWidget::new(progress_bar).erased()
}
"Prose" => {
//let label_text:&str = comp.get_bool( 1.or("checked") ).unwrap_or( false );
let label_text = "unnamed";

//read 'clip' property
let clip = false;

let prose = Prose::new(label_text).with_clip(clip);
NewWidget::new(prose).erased()
}
"ResizeObserver" => {
//check one child
let resize_observer = ResizeObserver::new( NewWidget::new(Label::new("")) );
NewWidget::new(resize_observer).erased()
}
"SizedBox" => {
let sized_box = SizedBox::new( NewWidget::new(Label::new("")) );
//size
NewWidget::new(sized_box).erased()
}
"Slider" => {
let min = 0.;
let max = 1.;
let value = 0.;
let step = 0.1;
let slider = Slider::new(min,max,value);
NewWidget::new(slider).erased()
}
"Spinner" => {
NewWidget::new(Spinner::new( )).erased()
}
"Split" => {
if comp.children.len() != 2 {
return Err(Error::ExactlyTwoChildRequired)
}
let split = Split::new(
build_widget_recurr(&comp.children[0], None)?,
build_widget_recurr(&comp.children[1], None)?
);
NewWidget::new(split).erased()
}
"TextArea" => {
let text = "";
let editable = false;
if editable {
let mut text_area = TextArea::<true>::new(text);
//text_area = text_area.with_word_wrap( false );
//text_area = text_area.with_text_alignment( TextAlign:: );
//text_area = text_area.set_hint( false );
//text_area = text_area.with_insert_newline( InsertNewLine:: );
NewWidget::new(text_area).erased()
} else {
let text_area = TextArea::<false>::new(text);
NewWidget::new(text_area).erased()
}
}
"TextInput" => {
let clip = true;
let text_align = TextAlign::Start;
let place_holder = "";
let mut text_input = TextInput::new("");
text_input = text_input.with_placeholder(place_holder);
text_input = text_input.with_clip(clip);
text_input = text_input.with_text_alignment(text_align);
NewWidget::new(text_input).erased()
}
"VariableLabel" => {

 */


#[derive(Debug,Clone)]
pub enum ValueConvError {
    InvalidType,
    InvalidValue,
    MandatoryParamMissing
}

impl ValueConvError {
    pub fn specific(self, idx:usize, key:&'static str) -> ArgumentError {
        ArgumentError {idx, key, err:self}
    }
}


pub trait FromValue: Sized {
    fn from_value(v:&Value) -> Result<Self, ValueConvError>;
}

macro_rules! impl_from_value {
    ( $st:ident { $($name:ident),* } ) => {
        impl FromValue for $st {
            fn from_value(v: &Value) -> Result<Self, ValueConvError> {
                if let Some(str) = v.as_str() {
                    let v = match str {
                        $(
                        stringify!($name) => Self::$name,
                        )*
                        _ => return Err(ValueConvError::InvalidValue)
                    };
                    Ok(v)
                } else {
                    Err(ValueConvError::InvalidType)
                }
            }
        }
    };
    ( $st:ident, { $($name:literal => $map:expr),* } ) => {
        impl FromValue for $st {
            fn from_value(v: &Value) -> Result<Self, ValueConvError> {
                if let Some(str) = $value.as_str() {
                    let v = match str {
                        $(
                        $name => $map,
                        )*
                        _ => return Err(ValueConvError::InvalidValue)
                    };
                    Ok(v)
                } else {
                    Err(ValueConvError::InvalidType)
                }
            }
        }
    };
}

impl_from_value!(Axis { Horizontal, Vertical } );
impl_from_value!(MainAxisAlignment { Start, Center, End, SpaceBetween, SpaceAround, SpaceEvenly } );
impl_from_value!(CrossAxisAlignment { Start, Center, End, Stretch } );
impl_from_value!(UnitPoint { TOP_LEFT, TOP, TOP_RIGHT, LEFT, CENTER, RIGHT, BOTTOM_LEFT, BOTTOM, BOTTOM_RIGHT } );


#[derive(Debug,Clone)]
pub struct ArgumentError {
    pub idx:usize,
    pub key:&'static str,
    pub err:ValueConvError,
}

pub struct ParamsStack<'a> {
    caller: Option<&'a Parameters>,
    current: &'a Parameters,
}

impl<'a> ParamsStack<'a> {
    pub fn new(caller:Option<&'a Parameters>, current:&'a Parameters) -> Self {
        Self {caller, current}
    }
}

pub trait FromParams : Sized {
    fn from_params(params:&ParamsStack) -> Result<Self,ArgumentError>;
}

macro_rules! impl_from_params {
    ( $st:ident, $(MUST [ $($name:ident:$name_ty:ty),* ])? $(,)? $(OPTION [$($opt_name:ident:$opt_ty:ty),* ])? ) => {
        pub struct $st {
            $($(pub $name:$name_ty,)*)?
            $($(pub $opt_name:Option<$opt_ty>,)*)?
        }

        impl FromParams for $st {
            fn from_params(params:&ParamsStack) -> Result<Self,ArgumentError> {
                match params.current {
                    Parameters::Args(args) => {
                        let mut cnt = 0;
                        let mut iter = args.iter();
                        $(
                        $(
                        let value = iter.next().ok_or( ArgumentError{err:ValueConvError::MandatoryParamMissing, idx:cnt, key:stringify!($name)})?;
                        let $name = <$name_ty as FromValue>::from_value(value).map_err(|e| e.specific(cnt, stringify!($name)))?;
                        cnt += 1;
                        )*
                        )?

                        $(
                        $(
                        let $opt_name = if let Some(value) = iter.next() {
                            Some( <$opt_ty as FromValue>::from_value(value).map_err(|e| e.specific(cnt, stringify!($opt_name)))? )
                        } else {
                            None
                        };
                        cnt += 1;
                        )*
                        )?
                        Ok( Self { $($($name,)*)? $($($opt_name,)*)? } )
                    }
                    Parameters::Map(map) => {
                        Err( ArgumentError {
                            err:ValueConvError::MandatoryParamMissing,
                            idx:0, key:""
                        } )
                    }
                }
            }
        }
    };
}

impl_from_params!(AlignArgs, MUST[unit_point: UnitPoint] );
impl_from_params!(FlexArgs,
    MUST [ axis: Axis ],
    OPTION [ main_axis_alignment: MainAxisAlignment,cross_axis_alignment: CrossAxisAlignment ] );