use std::{collections::HashMap, hash::Hash};

trait DrawableScreen {
    fn draw(&self) -> Result<()>;
}

trait DrawableButton {
    fn draw(&self) -> Result<()>;
}

trait Pressable {
    fn on_press(&self) -> Result<()>;
}

trait Rotatable {
    fn on_rotate(&self) -> Result<()>;
}

struct SplitButtonScreen {
    buttons: HashMap<loupedeck::Button, Box<dyn DrawableButton>>,
}

impl DrawableScreen for SplitButtonScreen {
    fn draw(&self) -> Result<()> {
        Ok(())
    }
}

struct Page {
    id: usize,
    name: String,
    screen: ScreenHandler,
}
