use crate::event::StaticEvent;

pub trait EventProcessor {
    fn process_events(&mut self, events: &mut Vec<StaticEvent>);
    fn process_transition(&mut self, transition: &mut StaticEvent);
}


