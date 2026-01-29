use std::str::FromStr;
use masonry::layout::Length;
use masonry::peniko::color::{AlphaColor, Srgb};
use masonry::properties::{ActiveBackground, Background, BorderColor, BorderWidth, ContentColor, DisabledBackground, DisabledContentColor, Gap, Padding};
use skui::{CssValue, Style, StyleProperty};
use masonry::core::StyleProperty as MasonryStyleProperty;
use masonry::parley::LineHeight;
use skui::selector::PseudoClass;

pub fn to_color_from_value(value:CssValue) -> Option<AlphaColor<Srgb>> {
    let v = match value {
        CssValue::HexColor(col) => AlphaColor::from_str(col).ok()?,
        CssValue::Rgb( (r,g,b) )  => AlphaColor::from_rgb8( r, g, b ),
        CssValue::Rgba( (r,g,b,a) ) => AlphaColor::from_rgba8( r, g, b, a ),
        CssValue::Ident( str ) => AlphaColor::from_str(str).ok()?,
        _ => return None
    };
    Some(v)
}

pub fn to_color(prop:&StyleProperty) -> Option<AlphaColor<Srgb>> {
    to_color_from_value( *prop.values.get(0)? )
}

pub fn to_background(prop:&StyleProperty) -> Option<Background> {
    Some( Background::Color( to_color( prop )? ) )
}

pub fn to_content_color(prop:&StyleProperty) -> Option<ContentColor> {
    Some( ContentColor::new( to_color( prop )?  ) )
}

pub fn to_border(prop:&StyleProperty) -> (Option<BorderWidth>, Option<BorderColor>) {
    let (width, color) = match &prop.values.as_slice() {
        &[width, CssValue::Ident(_brush), color] => {
            (width.as_f64(), to_color_from_value(*color))
        }
        &[width, color] => {
            (width.as_f64(), to_color_from_value(*color))
        }
        _ => (None, None)
    };
    (width.map( |v| BorderWidth::all(v)), color.map(|v| BorderColor::new(v)))
}

pub fn to_font_size(prop:&StyleProperty) -> Option<MasonryStyleProperty> {
    Some(
        MasonryStyleProperty::FontSize( prop.values.get(0)?.as_f64()? as _ )
    )
}

pub fn to_lineheight(prop:&StyleProperty) -> Option<MasonryStyleProperty> {
    let v = match prop.values.get(0)? {
        CssValue::Number(v) => LineHeight::FontSizeRelative( *v as _ ),
        CssValue::Px(v) => LineHeight::Absolute( *v as _ ),
        CssValue::Percent(v) => LineHeight::MetricsRelative( *v as _ ),
        _ => return None
    };
    Some(
        MasonryStyleProperty::LineHeight( v )
    )
}

pub fn style_parse(style:&Style, props:&mut masonry::core::Properties, styles:&mut Vec<MasonryStyleProperty>) {
    style.properties.iter().for_each( |property| {
        match property.key {
            //properties
            "border" => if let ( (w,c) ) = to_border( property ) {
                if let Some(w) = w { props.insert(w); }
                if let Some(c) = c { props.insert(c); }
            }
            "padding" => if let Some(v) = property.as_f64() {
                props.insert(Padding::all(v));
            }
            "gap" => if let Some(v) = property.as_f64() {
                props.insert( Gap::from(Length::px(v as _)) );
            },
            "background-color" => {
                if let Some(v) = to_background( property ) {
                    match style.selector.get_pseudo_class() {
                        Some(PseudoClass::Active) => { props.insert( ActiveBackground(v) ); }
                        Some(PseudoClass::Disabled) => { props.insert( DisabledBackground(v) ); }
                        _ => { props.insert( v ); }
                    };
                }
            },
            "color" => if let Some(v) = to_content_color(property) {
                match style.selector.get_pseudo_class() {
                    Some(PseudoClass::Disabled) => { props.insert( DisabledContentColor(v) ); },
                    _ => {props.insert( v ); },
                }
            }

            //style property
            "font-size" => if let Some(v) = to_font_size(property) {
                styles.push( v );
            }
            "line-height" => if let Some(v) = to_lineheight(property) {
                styles.push( v );
            }
            _ => eprintln!("Unknown style property : {}", property.key)
        }
    });
}