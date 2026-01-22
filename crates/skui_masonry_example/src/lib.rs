use masonry::core::{AsDynWidget, NewWidget, NoAction, Widget, WidgetPod};
use masonry::layout::Length;
use masonry::TextAlign;
use masonry::widgets::{Button, Checkbox, Flex, Grid, IndexedStack, Label, Passthrough, Portal, ProgressBar, Prose, ResizeObserver, SizedBox, Slider, Spinner, Split, TextArea, TextInput, VariableLabel};
use masonry_winit::app::{AppDriver, NewWindow};
use skui::{Component, Parameters, ParseError, SKUIParseError, SKUI};
mod editor;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug,Clone)]
pub enum Error {
    RootComponentNotFound,
    UnknownComponent(String),
    RequiredChildren(usize),
    AtLeastOneRequired,
    ExactlyTwoChildRequired,
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

pub fn build_root_widget(src:&str) -> Result<NewWidget<impl Widget + ?Sized >> {
    let skui = SKUI::parse( src )?;
    let main_comp = get_main_component( &skui )?;
    let widget = build_widget_recurr(&main_comp, None);
    skui.components.len();
    widget
}

fn build_widget_recurr(comp:&Component, params:Option<&Parameters>) -> Result<NewWidget<impl Widget + ?Sized + use<>>> {
    let v = match comp.name.as_str() {
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
            let text = "";
            let var_label = VariableLabel::new(text);
            NewWidget::new(var_label).erased()
        }
        name @ _ => {
            //check custom components

            return Err(Error::UnknownComponent(name.to_string()))
        },
    };
    Ok(v)
}
