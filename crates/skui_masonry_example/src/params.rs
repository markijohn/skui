use masonry::kurbo::Axis;
use masonry::layout::UnitPoint;
use masonry::properties::types::{CrossAxisAlignment, MainAxisAlignment};
use masonry::TextAlign;
use masonry::widgets::{FlexBasis, InsertNewline};
use skui::{Component, Number, Parameters, Value};

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


pub trait FromValue<'a>: Sized {
    fn from_value(v:&'a Value) -> Result<Self, ValueConvError>;
}

impl <'a> FromValue<'a> for String {
    fn from_value(v:&'a Value) -> Result<String, ValueConvError> {
        Ok( v.as_str().ok_or(ValueConvError::InvalidType)?.to_string() )
    }
}

impl <'a> FromValue<'a> for &'a str {
    fn from_value(v:&'a Value) -> Result<&'a str, ValueConvError> {
        Ok( v.as_str().ok_or(ValueConvError::InvalidType)? )
    }
}

impl <'a> FromValue<'a> for bool {
    fn from_value(v:&'a Value) -> Result<bool, ValueConvError> {
        Ok( v.as_bool().ok_or(ValueConvError::InvalidType)? )
    }
}

impl <'a> FromValue<'a> for Value {
    fn from_value(v:&'a Value) -> Result<Self, ValueConvError> {
        Ok( v.clone() )
    }
}

impl <'a> FromValue<'a> for Number {
    fn from_value(v:&'a Value) -> Result<Number, ValueConvError> {
        if let Value::Number(num) = v {
            Ok(num.clone())
        } else {  Err(ValueConvError::InvalidType) }
    }
}


impl <'a> FromValue<'a> for &'a Component {
    fn from_value(v:&'a Value) -> Result< Self, ValueConvError> {
        if let Value::Component(comp) = v {
            Ok( comp )
        } else {  Err(ValueConvError::InvalidType) }
    }
}

macro_rules! impl_from_value {
    ( I64 { $($name:ident),* } ) => {
        $(
        impl <'a> FromValue<'a>  for $name {
            fn from_value(v:&Value) -> Result<Self, ValueConvError> {
                Ok( v.as_i64().ok_or(ValueConvError::InvalidType)? as _ )
            }
        }
        )*
    };
    ( F64 { $($name:ident),* } ) => {
        $(
        impl <'a> FromValue<'a>  for $name {
            fn from_value(v:&Value) -> Result<Self, ValueConvError> {
                Ok( v.as_f64().ok_or(ValueConvError::InvalidType)? as _ )
            }
        }
        )*
    };
    ( $st:ident { $($name:ident),* } ) => {
        impl <'a> FromValue<'a>  for $st {
            fn from_value(v: &'a Value) -> Result<Self, ValueConvError> {
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
        impl <'a> FromValue<'a>  for $st {
            fn from_value(v: &'a Value) -> Result<Self, ValueConvError> {
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

impl_from_value!(I64 {u8, i8, i32, u32, i64, u64, isize, usize} );
impl_from_value!(F64 {f32, f64} );
impl_from_value!(Axis { Horizontal, Vertical } );
impl_from_value!(MainAxisAlignment { Start, Center, End, SpaceBetween, SpaceAround, SpaceEvenly } );
impl_from_value!(CrossAxisAlignment { Start, Center, End, Stretch } );
impl_from_value!(UnitPoint { TOP_LEFT, TOP, TOP_RIGHT, LEFT, CENTER, RIGHT, BOTTOM_LEFT, BOTTOM, BOTTOM_RIGHT } );
impl_from_value!(FlexBasis { Auto, Zero} );
impl_from_value!(TextAlign {Start,End,Left,Center,Right,Justify} );
impl_from_value!(InsertNewline {OnEnter, OnShiftEnter, Never});

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

pub trait FromParams<'a> : Sized {
    fn from_params(params:&'a ParamsStack) -> Result<Self,ArgumentError>;
}

macro_rules! impl_from_params {
    ( $st:ident $(<$lt:lifetime>)? , $(MUST [ $($name:ident:$name_ty:ty),* ])? $(,)? $(OPTION [$($opt_name:ident:$opt_ty:ty),* ])? ) => {
        pub struct $st $(<$lt>)? {
            $($(pub $name:$name_ty,)*)?
            $($(pub $opt_name:Option<$opt_ty>,)*)?
        }

        impl <'a> FromParams<'a> for $st $(<$lt>)? {
            fn from_params(params:&'a ParamsStack) -> Result<Self,ArgumentError> {
                match params.current {
                    Parameters::Args(args) => {
                        let mut cnt = 0;
                        let mut iter = args.iter();
                        $(
                        $(
                        let value = iter.next().ok_or( ArgumentError{err:ValueConvError::MandatoryParamMissing, idx:cnt, key:stringify!($name)})?;
                        let $name = <$name_ty as FromValue<'a>>::from_value(value).map_err(|e| e.specific(cnt, stringify!($name)))?;
                        cnt += 1;
                        )*
                        )?

                        $(
                        $(
                        let $opt_name = if let Some(value) = iter.next() {
                            Some( <$opt_ty as FromValue<'a>>::from_value(value).map_err(|e| e.specific(cnt, stringify!($opt_name)))? )
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
impl_from_params!(ButtonArgs, MUST[text:String]);
impl_from_params!(CheckboxArgs, MUST[text:String], OPTION [checked:bool] );
impl_from_params!(FlexArgs, MUST [ axis: Axis ], OPTION [ main_axis_alignment: MainAxisAlignment,cross_axis_alignment: CrossAxisAlignment ] );
impl_from_params!(FlexItemArgs <'a>, MUST[comp:&'a Component,flex:f64], OPTION[basis:FlexBasis,alignment:CrossAxisAlignment] );
impl_from_params!(FlexSpacerArgs, MUST[value:Number]);
impl_from_params!(GridArgs, MUST[x:i32, y:i32] );
impl_from_params!(GridParamsArgs<'a>, MUST[comp:&'a Component, x:i32, y:i32], OPTION[w:i32, h:i32] );
impl_from_params!(IndexedStackArgs, MUST[index:usize]);
impl_from_params!(LabelArgs, MUST[text:String] );
impl_from_params!(ProseArgs, MUST[text:String], OPTION[clip:bool] );
impl_from_params!(PassthroughArgs<'a>, MUST[comp:&'a Component]);
impl_from_params!(PortalArgs<'a>, MUST[comp:&'a Component]);
impl_from_params!(ProgressBarArgs, OPTION[progress:f64]);
impl_from_params!(ResizeObserverArgs<'a>, MUST[comp:&'a Component]);
impl_from_params!(SizedBoxArgs<'a>, MUST[comp:&'a Component], OPTION[width:f64, height:f64]);
impl_from_params!(SliderArgs, MUST[min:f64,max:f64,value:f64], OPTION[step:f64] );
impl_from_params!(SplitArgs<'a>, OPTION[first:&'a Component,second:&'a Component] );
impl_from_params!(TextAreaArgs<'a>, OPTION[text:&'a str,alignment:TextAlign,insert_newline:InsertNewline,hint:bool,editable:bool]);
impl_from_params!(TextInputArgs<'a>, OPTION[placeholder:&'a str, text:&'a str,clip:bool,alignment:TextAlign] );