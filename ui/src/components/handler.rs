pub trait ClickHandler {
    fn handle_click(&mut self);
}

impl<F> ClickHandler for F
where F: FnMut()
{
    fn handle_click(&mut self) {
        (self)()
    }
}
