use std::{
    cell::RefCell,
    fmt::{Debug, Formatter},
    rc::Rc,
};

use clipboard::{ClipboardContext, ClipboardProvider};

#[derive(Clone)]
pub struct Clipboard {
    clipboard: Rc<RefCell<ClipboardContext>>,
}

impl Clipboard {
    pub fn initialize() -> anyhow::Result<Self> {
        Ok(Self {
            clipboard: Rc::new(RefCell::new(
                ClipboardContext::new().map_err(|err| anyhow::anyhow!("{}", err))?,
            )),
        })
    }

    pub fn set_contents(&self, text: String) -> anyhow::Result<()> {
        self.clipboard
            .borrow_mut()
            .set_contents(text)
            .map_err(|err| anyhow::anyhow!("{}", err))
    }

    pub fn get_contents(&self) -> anyhow::Result<String> {
        self.clipboard
            .borrow_mut()
            .get_contents()
            .map_err(|err| anyhow::anyhow!("{}", err))
    }
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").field("clipboard", &"...").finish()
    }
}
