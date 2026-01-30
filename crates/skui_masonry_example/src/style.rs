use std::str::FromStr;
use masonry::layout::Length;
use masonry::peniko::color::{AlphaColor, Srgb};
use masonry::properties::{ActiveBackground, Background, BorderColor, BorderWidth, ContentColor, DisabledBackground, DisabledContentColor, FocusedBorderColor, Gap, HoveredBorderColor, Padding};
use skui::{CssValue, Style, StyleProperty};
use masonry::core::StyleProperty as MasonryStyleProperty;
use masonry::parley::LineHeight;
use skui::selector::PseudoClass;

pub fn to_color_from_value(value:CssValue) -> Option<AlphaColor<Srgb>> {
    let v = match value {
        CssValue::HexColor(col) => AlphaColor::from_str( &format!("#{col}") ).ok()?,
        CssValue::Rgb( (r,g,b) )  => AlphaColor::from_rgb8( r, g, b ),
        CssValue::Rgba( (r,g,b,a) ) => AlphaColor::from_rgba8( r, g, b, a ),
        CssValue::Ident( str ) => {
            AlphaColor::from_str(str).ok()?
        },
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

pub fn style_parse(build_prop:bool, build_styles:bool, style:&Style, props:&mut masonry::core::Properties, styles:&mut Vec<MasonryStyleProperty>) {
    style.properties.iter().for_each( |property| {
        let mut proc_property = build_prop;
        if build_prop {
            match property.key.trim() {
                //properties
                "border" => if let ( (w, c) ) = to_border(property) {
                    if let Some(w) = w { props.insert(w); }
                    if let Some(c) = c { props.insert(c); }
                }
                "border-width" => if let Some(v) = property.as_f64() {
                    props.insert(BorderWidth::all(v));
                }
                "border-color" => if let Some(v) = to_color(property) {
                    match style.selector.get_pseudo_class() {
                        Some(PseudoClass::Hover) => { props.insert(HoveredBorderColor(BorderColor::new(v))); }
                        Some(PseudoClass::Focus) => { props.insert(FocusedBorderColor(BorderColor::new(v))); }
                        None => { props.insert(BorderColor::new(v)); }
                        v @ _ => { eprintln!("Unknown border-color pseudo state : {v:?}"); }
                    };
                }
                "padding" => if let Some(v) = property.as_f64() {
                    props.insert(Padding::all(v));
                }
                "gap" => if let Some(v) = property.as_f64() {
                    props.insert(Gap::from(Length::px(v as _)));
                },
                "background-color" => {
                    if let Some(v) = to_background(property) {
                        match style.selector.get_pseudo_class() {
                            Some(PseudoClass::Active) => { props.insert(ActiveBackground(v)); }
                            Some(PseudoClass::Disabled) => { props.insert(DisabledBackground(v)); }
                            None => { props.insert(v); }
                            v @ _ => { eprintln!("Unknown background-color state : {v:?}"); }
                        };
                    }
                },
                "color" => if let Some(v) = to_content_color(property) {
                    match style.selector.get_pseudo_class() {
                        Some(PseudoClass::Disabled) => { props.insert(DisabledContentColor(v)); },
                        _ => { props.insert(v); },
                    }
                }
                _ => {
                    proc_property = false;
                }
            }
        }

        if !proc_property && build_styles {
            match property.key {
                //style property
                "font-size" => if let Some(v) = to_font_size(property) {
                    styles.push( v );
                }
                "line-height" => if let Some(v) = to_lineheight(property) {
                    styles.push( v );
                }
                _ => {
                    if !proc_property {
                        eprintln!("Unknown style property : {}", property.key)
                    }
                }
            }
        }

    });
}