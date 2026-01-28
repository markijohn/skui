use masonry::kurbo::Axis;
use masonry::layout::UnitPoint;
use masonry::properties::types::{CrossAxisAlignment, MainAxisAlignment};
use masonry::TextAlign;
use masonry::widgets::{FlexBasis, InsertNewline};
use skui::{Component, Number, Parameters, Value, SKUI};

#[derive(Debug,Clone)]
pub enum ValueConvError {
    InvalidType,
    InvalidValue,
    MandatoryParamMissing
}

impl ValueConvError {
    pub fn specific(self, func:&str, comp:&str, idx:usize, key:&'static str) -> ArgumentError {
        ArgumentError {func:func.into(), comp:comp.into(), idx, key, err:self}
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

impl <'a> FromValue<'a> for Value<'a> {
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


impl <'a> FromValue<'a> for &'a Component<'a> {
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
    pub func:String,
    pub comp:String,
    pub idx:usize,
    pub key:&'static str,
    pub err:ValueConvError,
}

// Search for the value in the current parameter. If the value is “Relative”, search in the caller parameter.
#[derive(Debug,Clone)]
pub struct ParamsStack<'a> {
    pub fn_name : &'a str,
    pub params_stack : Vec<&'a Parameters<'a>>,
    pub wrap_id : Option<&'a str>,
    pub wrap_classes : Option<&'a [&'a str]>,
    pub component: &'a Component<'a>,
    pub skui: &'a SKUI<'a>,
}


const MAIN_COMPONENT_NAME: &'static str = "Main";

impl<'a> ParamsStack<'a> {

    pub fn new_main(param:&'a Parameters<'a>, skui:&'a SKUI<'a>) -> Option<Self> {
        let main_comp = &skui.get_root_component(MAIN_COMPONENT_NAME)?.component;
        Some( Self {
            fn_name: MAIN_COMPONENT_NAME,
            component: main_comp,
            params_stack:vec![param],
            wrap_id:None, //for extern caller
            wrap_classes:None, //for extern caller
            skui
        } )
    }

    pub fn new_stack(&self, comp:&'a Component<'a>) -> Self {

        //This component is caller root component
        if let Some(root_comp) = self.skui.get_root_component(comp.name) {
            let root_lookup_comp = &root_comp.component;
            let mut stack = self.params_stack.clone();
            println!("CALLL PARAM {:?}", comp.params);
            stack.push(&comp.params);
            let wrap_classes = if comp.classes.len() > 0 {
                Some(comp.classes.as_slice())
            } else { None };
            Self {
                fn_name : root_comp.name, //== comp.name
                params_stack : stack,
                wrap_id : comp.id,
                wrap_classes,
                component : root_lookup_comp,
                skui : self.skui
            }
        } else {
            let stack = self.params_stack.clone();
            Self {
                fn_name : self.fn_name,
                params_stack : stack,
                wrap_id : None,
                wrap_classes : None,
                component: comp,
                skui : self.skui
            }
        }

    }

    pub fn get_id(&self) -> Option<&'a str> {
        self.wrap_id.or( self.component.id )
    }

    pub fn get_classes(&self) -> impl Iterator<Item=&'a str> {
        self.wrap_classes.unwrap_or( &[] ).iter().chain( self.component.classes.iter() ).copied()
    }

    pub fn get(&self, idx:usize, key:&'a str) -> Option<&'a Value<'a>> {
        let mut curr_val:Option<&'a Value<'a>> = None;

        for stack in std::iter::once(&self.component.params).chain( self.params_stack.iter().rev().copied() ) {
            if let Some(Value::Relative( key)) = curr_val {
                let value = stack.get_as_rk( key.as_slice() );
                if let Some(v) = value {
                    if let Value::Relative(_) = v {
                        curr_val = value;
                    } else {
                        return value;
                    }
                } else {
                    return value;
                }
            } else {
                let v = stack.get(idx, key);
                if let Some(Value::Relative(_)) = v {
                    curr_val = v;
                } else {
                    return v
                }
            }
        }
        curr_val
    }

    pub fn children(&self) -> impl Iterator<Item=&'a Component<'a>> {
        self.component.children.iter()
    }
}

pub trait FromParams<'a> : Sized {
    fn from_params(params:&'a ParamsStack) -> Result<Self,ArgumentError>;
}


#[macro_export]
macro_rules! impl_from_params {
    ( $st:ident $(<$lt:lifetime>)? , $(MUST [ $($name:ident:$name_ty:ty),* ])? $(,)? $(OPTION [$($opt_name:ident:$opt_ty:ty),* ])? ) => {
        #[derive(Debug,Clone)]
        pub struct $st $(<$lt>)? {
            $($(pub $name:$name_ty,)*)?
            $($(pub $opt_name:Option<$opt_ty>,)*)?
        }

        impl <'a> FromParams<'a> for $st $(<$lt>)? {
            fn from_params(params:&'a ParamsStack) -> Result<Self,ArgumentError> {

                let mut cnt = 0;
                $(
                $(
                let value = params.get(cnt, stringify!($name)).ok_or( ArgumentError{err:ValueConvError::MandatoryParamMissing, func:params.fn_name.into(), comp:params.component.name.into(), idx:cnt, key:stringify!($name)})?;
                let $name = <$name_ty as FromValue<'a>>::from_value(value).map_err(|e| e.specific(params.fn_name, params.component.name, cnt, stringify!($name)))?;
                cnt += 1;
                )*
                )?

                $(
                $(
                let $opt_name = if let Some(value) = params.get(cnt, stringify!($opt_name)) {
                    Some( <$opt_ty as FromValue<'a>>::from_value(value).map_err(|e| e.specific(params.fn_name, params.component.name,cnt, stringify!($opt_name)))? )
                } else {
                    None
                };
                cnt += 1;
                )*
                )?
                Ok( Self { $($($name,)*)? $($($opt_name,)*)? } )

            }
        }
    };
}

impl_from_params!(AlignArgs<'a>, MUST[unit_point: UnitPoint, comp:&'a Component<'a>] );
impl_from_params!(ButtonArgs<'a>, MUST[text:&'a str]);
impl_from_params!(CheckboxArgs<'a>, MUST[text:&'a str], OPTION [checked:bool] );
impl_from_params!(FlexArgs, MUST [ axis: Axis ], OPTION [ main_axis_alignment: MainAxisAlignment,cross_axis_alignment: CrossAxisAlignment ] );
impl_from_params!(FlexItemArgs <'a>, MUST[comp:&'a Component<'a>,flex:f64], OPTION[basis:FlexBasis,alignment:CrossAxisAlignment] );
impl_from_params!(FlexSpacerArgs, MUST[value:Number]);
impl_from_params!(GridArgs, MUST[x:i32, y:i32] );
impl_from_params!(GridParamsArgs<'a>, MUST[comp:&'a Component<'a>, x:i32, y:i32], OPTION[w:i32, h:i32] );
impl_from_params!(IndexedStackArgs, MUST[index:usize]);
impl_from_params!(LabelArgs<'a>, MUST[text:&'a str] );
impl_from_params!(ProseArgs<'a>, MUST[text:&'a str], OPTION[clip:bool] );
impl_from_params!(PassthroughArgs<'a>, MUST[comp:&'a Component<'a>]);
impl_from_params!(PortalArgs<'a>, MUST[comp:&'a Component<'a>]);
impl_from_params!(ProgressBarArgs, OPTION[progress:f64]);
impl_from_params!(ResizeObserverArgs<'a>, MUST[comp:&'a Component<'a>]);
impl_from_params!(SizedBoxArgs<'a>, MUST[comp:&'a Component<'a>], OPTION[width:f64, height:f64]);
impl_from_params!(SliderArgs, MUST[min:f64,max:f64,value:f64], OPTION[step:f64] );
impl_from_params!(SplitArgs<'a>, OPTION[first:&'a Component<'a>,second:&'a Component<'a>] );
impl_from_params!(TextAreaArgs<'a>, OPTION[text:&'a str,alignment:TextAlign,insert_newline:InsertNewline,hint:bool,editable:bool]);
impl_from_params!(TextInputArgs<'a>, OPTION[placeholder:&'a str, text:&'a str,clip:bool,alignment:TextAlign] );
impl_from_params!(VariableLabelArgs<'a>, MUST[text:&'a str]);