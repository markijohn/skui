use masonry::core::{AsDynWidget, NewWidget, NoAction, Widget, WidgetPod};
use masonry::widgets::{Button, Flex, Label, SizedBox};
use masonry_winit::app::{AppDriver, NewWindow};
use skui::{Component, ParseError, SKUI};
mod editor;

pub struct SKUIMasonry {
    source: String,
    skui: SKUI,
}



impl SKUIMasonry {
    pub fn new(source:String) -> Result<SKUIMasonry, ParseError> {
        let skui = SKUI::parse( source.as_str() )?;
        Ok( Self {
            source,
            skui
        } )
    }

    fn get_main_component(&self) -> Option<&Component> {
        let skui = &self.skui;
        let none_id_comp = skui.components.iter().find( |&c| c.id.is_none() );
        let main_id_comp = skui.components.iter().find( |&c| c.id.as_ref().map(String::as_str).unwrap_or("") == "main" );
        none_id_comp.or(main_id_comp)
    }

    fn build_main_widget(&self) -> Option<NewWidget<impl Widget + ?Sized>> {
        self.get_main_component()
            .and_then( |main_comp| self.build_widget(&main_comp) )
    }

    fn build_widget(&self, comp:&Component) -> Option<NewWidget<impl Widget + ?Sized>> {
        let v = match comp.name.as_str() {
            "Flex" => {
                let mut flex = Flex::row();

                NewWidget::new(flex).erased()
            }
            "Button" => {
                let btn = Button::new( NewWidget::new(Label::new("BUTTON")) );

                NewWidget::new(btn).erased()
            }
            name @ _ => panic!("{} is not implemented yet", name),
        };
        Some(v)
    }
}