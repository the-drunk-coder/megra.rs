use crate::event::Event;

pub trait EventProcessor {
    fn process_events(&mut self, events: &mut Vec<Event>);
    fn process_transition(&mut self, transition: &mut Event);
}


