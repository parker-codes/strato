use crate::game::{GameContext, GameState};

pub struct Subscriber<'s>(Box<dyn Fn(SubscriberEvent) + 's>);

impl std::fmt::Debug for Subscriber<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subscriber")
    }
}

impl<'s> Subscriber<'s> {
    pub fn new<F: Fn(SubscriberEvent) + 's>(f: F) -> Self {
        Self(Box::new(f))
    }

    pub fn emit(&self, event: SubscriberEvent) {
        (self.0)(event)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubscriberEvent<'a> {
    StateChanged(&'a GameState),
    ContextChanged(&'a GameContext),
}

pub trait Subscribe<'s> {
    fn subscribe(&mut self, f: impl Fn(SubscriberEvent) + 's);
    fn unsubscribe(&mut self);
    fn notify(&self, event: SubscriberEvent);
}
