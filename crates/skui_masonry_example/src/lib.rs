use masonry::core::{AsDynWidget, NewWidget, NoAction, Widget, WidgetPod};
use masonry::widgets::{Button, Checkbox, Flex, Grid, IndexedStack, Label, Passthrough, Prose, ResizeObserver, SizedBox};
use masonry_winit::app::{AppDriver, NewWindow};
use skui::{Component, ParseError, SKUIParseError, SKUI};
mod editor;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug,Clone)]
pub enum Error {
    RootComponentNotFound,
    UnknownComponent(String),
    RequiredChildren(usize),
    ParseError(SKUIParseError)
}

impl From<SKUIParseError> for Error {
    fn from(e:SKUIParseError) -> Self {
        Error::ParseError(e)
    }
}

fn get_main_component(skui:&SKUI) -> Result<&Component> {
    let none_id_comp = skui.components.iter().find( |&c| c.id.is_none() );
    let main_id_comp = skui.components.iter().find( |&c| c.id.as_ref().map(String::as_str).unwrap_or("") == "main" );
    none_id_comp.or(main_id_comp).ok_or_else(|| Error::RootComponentNotFound)
}

pub fn build_root_widget<'a>(src:&'a str) -> Result<NewWidget<impl Widget + ?Sized>> {
    let skui = SKUI::parse(src)?;
    let main_comp = get_main_component(&skui)?;
    get_main_component( &skui )
        .and_then( |main_comp| build_widget_recurr(&main_comp) )
}

fn build_widget_recurr(comp:&Component) -> Result<NewWidget<impl Widget + ?Sized>> {
    let v = match comp.name.as_str() {
        "Flex" => {
            let mut flex = Flex::row();

            //Consume children

            NewWidget::new(flex).erased()
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
        "SizedBox" => {

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
            todo!()
        }
        "ProgressBar" => {
            todo!()
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
        name @ _ => {
            //check custom components

            return Err(Error::UnknownComponent(name.to_string()))
        },
    };
    Ok(v)
}
