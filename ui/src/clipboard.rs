use clipboard::{ClipboardContext, ClipboardProvider};
use std::cell::RefCell;
use std::rc::Rc;

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

    pub fn get_contents(&self) -> anyhow::Result<String> {
        self.clipboard
            .borrow_mut()
            .get_contents()
            .map_err(|err| anyhow::anyhow!("{}", err))
    }
}
