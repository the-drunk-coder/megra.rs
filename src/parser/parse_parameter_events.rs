use nom::{
    branch::alt,
    bytes::complete::tag,        
    combinator::map,
    error::VerboseError,
    IResult, 
};

use crate::builtin_types::*;
use crate::event::EventOperation;

pub fn parse_level_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {    
    alt((		
	map(tag("lvl"), |_| BuiltIn::ModEvent(BuiltInEvent::Level(EventOperation::Replace))),	
	map(tag("lvl-add"), |_| BuiltIn::ModEvent(BuiltInEvent::Level(EventOperation::Add))),
	map(tag("lvl-mul"), |_| BuiltIn::ModEvent(BuiltInEvent::Level(EventOperation::Multiply))),
	map(tag("lvl-sub"), |_| BuiltIn::ModEvent(BuiltInEvent::Level(EventOperation::Subtract))),
	map(tag("lvl-div"), |_| BuiltIn::ModEvent(BuiltInEvent::Level(EventOperation::Divide))),	
    ))(i)
}

pub fn parse_duration_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {    
    alt((		
	map(tag("dur"), |_| BuiltIn::ModEvent(BuiltInEvent::Duration(EventOperation::Replace))),	
	map(tag("dur-add"), |_| BuiltIn::ModEvent(BuiltInEvent::Duration(EventOperation::Add))),
	map(tag("dur-mul"), |_| BuiltIn::ModEvent(BuiltInEvent::Duration(EventOperation::Multiply))),
	map(tag("dur-sub"), |_| BuiltIn::ModEvent(BuiltInEvent::Duration(EventOperation::Subtract))),
	map(tag("dur-div"), |_| BuiltIn::ModEvent(BuiltInEvent::Duration(EventOperation::Divide))),	
    ))(i)
}

pub fn parse_reverb_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {    
    alt((		
	map(tag("rev"), |_| BuiltIn::ModEvent(BuiltInEvent::Reverb(EventOperation::Replace))),	
	map(tag("rev-add"), |_| BuiltIn::ModEvent(BuiltInEvent::Reverb(EventOperation::Add))),
	map(tag("rev-mul"), |_| BuiltIn::ModEvent(BuiltInEvent::Reverb(EventOperation::Multiply))),
	map(tag("rev-sub"), |_| BuiltIn::ModEvent(BuiltInEvent::Reverb(EventOperation::Subtract))),
	map(tag("rev-div"), |_| BuiltIn::ModEvent(BuiltInEvent::Reverb(EventOperation::Divide))),	
    ))(i)
}
