/*
   Copyright 2019 Supercomputing Systems AG

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.

*/

use std::{
    collections::HashMap,
    convert::TryFrom,
    marker::PhantomData,
    str::FromStr,
};

use codec::alloc::string::FromUtf8Error;
use log::{debug, info};
use metadata::{DecodeDifferent, RuntimeMetadata, RuntimeMetadataPrefixed, EventMetadata};
use serde::{Deserialize, Serialize};

pub fn pretty_format(metadata: &RuntimeMetadataPrefixed) -> Result<String, FromUtf8Error> {
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    metadata.serialize(&mut ser).unwrap();
    String::from_utf8(ser.into_inner())
}

pub type NodeMetadata = Vec<Module>;

pub trait Print {
    fn print_events(&self);
    fn print_calls(&self);
}

impl Print for NodeMetadata {
    fn print_events(&self) {
        for m in self {
            m.print_events();
        }
    }

    fn print_calls(&self) {
        for m in self {
            m.print_calls()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Module {
    pub name: String,
    pub calls: Vec<Call>,
    pub events: HashMap<u8, Event>,
}

impl Module {
    fn new(name: &DecodeDifferent<&'static str, std::string::String>) -> Module {
        Module {
            name: format!("{:?}", name).replace("\"", ""),
            calls: Vec::<Call>::new(),
            events: HashMap::new(),
        }
    }

    pub fn print_events(&self) {
        println!(
            "----------------- Events for Module: {} -----------------\n",
            self.name
        );
        for e in &self.events {
            println!("{:?}", e);
        }
        println!()
    }

    pub fn print_calls(&self) {
        println!(
            "----------------- Calls for Module: {} -----------------\n",
            self.name
        );
        for e in &self.calls {
            println!("{:?}", e);
        }
        println!()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Call {
    pub name: String,
    pub args: Vec<Arg>,
}

impl Call {
    fn new(name: &DecodeDifferent<&'static str, std::string::String>) -> Call {
        Call {
            name: format!("{:?}", name).replace("\"", ""),
            args: Vec::<Arg>::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Event {
    pub name: String,
    // in this case the only the argument types are provided as strings
    pub arguments: Vec<EventArg>,
}

impl Event {
    pub fn arguments(&self) -> Vec<EventArg> {
        self.arguments.to_vec()
    }
}

impl Event {
    fn new(name: &DecodeDifferent<&'static str, std::string::String>) -> Event {
        Event { name: format!("{:?}", name).replace("\"", ""), arguments: Vec::<EventArg>::new() }
    }
}

/// Naive representation of event argument types, supports current set of substrate EventArg types.
/// If and when Substrate uses `type-metadata`, this can be replaced.
///
/// Used to calculate the size of a instance of an event variant without having the concrete type,
/// so the raw bytes can be extracted from the encoded `Vec<EventRecord<E>>` (without `E` defined).
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EventArg {
    Primitive(String),
    Vec(Box<EventArg>),
    Tuple(Vec<EventArg>),
}

impl FromStr for EventArg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("Vec<") {
            if s.ends_with('>') {
                Ok(EventArg::Vec(Box::new(s[4..s.len() - 1].parse()?)))
            } else {
                Err(Error::InvalidEventArg(
                    s.to_string(),
                    "Expected closing `>` for `Vec`",
                ))
            }
        } else if s.starts_with('(') {
            if s.ends_with(')') {
                let mut args = Vec::new();
                for arg in s[1..s.len() - 1].split(',') {
                    let arg = arg.trim().parse()?;
                    args.push(arg)
                }
                Ok(EventArg::Tuple(args))
            } else {
                Err(Error::InvalidEventArg(
                    s.to_string(),
                    "Expecting closing `)` for tuple",
                ))
            }
        } else {
            Ok(EventArg::Primitive(s.to_string()))
        }
    }
}

impl EventArg {
    /// Returns all primitive types for this EventArg
    pub fn primitives(&self) -> Vec<String> {
        match self {
            EventArg::Primitive(p) => vec![p.clone()],
            EventArg::Vec(arg) => arg.primitives(),
            EventArg::Tuple(args) => {
                let mut primitives = Vec::new();
                for arg in args {
                    primitives.extend(arg.primitives())
                }
                primitives
            }
        }
    }
}

fn convert<B: 'static, O: 'static>(dd: DecodeDifferent<B, O>) -> Result<O, Error> {
    match dd {
        DecodeDifferent::Decoded(value) => Ok(value),
        _ => Err(Error::ExpectedDecoded),
    }
}

fn convert_event(
    event: EventMetadata,
) -> Result<Event, Error> {
    let name = convert(event.name)?;
    let mut arguments = Vec::new();
    for arg in convert(event.arguments)? {
        let arg = arg.parse::<EventArg>()?;
        arguments.push(arg);
    }
    Ok(Event { name, arguments })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub ty: String,
}

impl Arg {
    fn new(
        name: &DecodeDifferent<&'static str, std::string::String>,
        ty: &DecodeDifferent<&'static str, std::string::String>,
    ) -> Arg {
        Arg {
            name: format!("{:?}", name).replace("\"", ""),
            ty: format!("{:?}", ty).replace("\"", ""),
        }
    }
}

pub fn parse_metadata(metadata: RuntimeMetadataPrefixed) -> Result<Vec<Module>, Error> {
    let mut mod_vec = Vec::<Module>::new();

    let meta = match metadata.1 {
        RuntimeMetadata::V8(meta) => meta,
        _ => return Err(Error::InvalidVersion),
    };
    debug!("-------------------- modules ----------------");
    for module in convert(meta.modules)?.into_iter() {
        debug!("module: {:?}", module.name);
        let mut _mod = Module::new(&module.name);
        debug!("-------------------- calls ----------------");
        if let Some(calls) = module.calls {
            let calls = convert(calls)?;

            if calls.is_empty() {
                // indices modules does for some reason list `Some([])' as calls and is thus counted in the call enum
                // there might be others doing the same.
                _mod.calls.push(Default::default())
            }

            for call in calls.into_iter() {
                let mut _call = Call::new(&call.name);
                for arg in convert(call.arguments)?.into_iter() {
                    _call.args.push(Arg::new(&arg.name, &arg.ty));
                }
                _mod.calls.push(_call);
            }
        }

        if let Some(events) = module.event {
            let mut event_map = HashMap::new();
            for (index, e) in convert(events)?.into_iter().enumerate() {
                event_map.insert(index as u8, convert_event(e).unwrap());
            }
            _mod.events = event_map;
        } else {
            debug!("No calls for this module");
        }

        mod_vec.push(_mod);
    }
    for m in &mod_vec {
        info!("{:?}", m);
    }
    debug!("successfully decoded metadata");
    Ok(mod_vec)
}

#[derive(Debug)]
pub enum Error {
    InvalidPrefix,
    InvalidVersion,
    ExpectedDecoded,
    InvalidEventArg(String, &'static str),
}
