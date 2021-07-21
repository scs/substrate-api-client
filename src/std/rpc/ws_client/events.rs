// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-subxt.
//
// subxt is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// subxt is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-subxt.  If not, see <http://www.gnu.org/licenses/>.

use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    marker::Send,
};

use codec::{Codec, Compact, Decode, Encode, Error as CodecError, Input, Output};
use sp_runtime::DispatchError;
use support::weights::DispatchInfo;
use system::Phase;

use crate::std::node_metadata::{EventArg, Metadata, MetadataError};
use crate::std::AccountId;
use crate::{Balance, BlockNumber, Hash, Moment};

/// Event for the System module.
#[derive(Clone, Debug, Decode)]
pub enum SystemEvent {
    /// An extrinsic completed successfully.
    ExtrinsicSuccess(DispatchInfo),
    /// An extrinsic failed.
    ExtrinsicFailed(DispatchError, DispatchInfo),
}

/// Top level Event that can be produced by a substrate runtime
#[derive(Debug)]
pub enum RuntimeEvent {
    System(SystemEvent),
    Raw(RawEvent),
}

/// Raw bytes for an Event
#[derive(Debug)]
pub struct RawEvent {
    /// The name of the module from whence the Event originated
    pub module: String,
    /// The name of the Event
    pub variant: String,
    /// The raw Event data
    pub data: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum EventsError {
    #[error("Scale codec error: {0:?}")]
    CodecError(#[from] CodecError),
    #[error("Metadata error: {0:?}")]
    Metadata(#[from] MetadataError),
    #[error("Type Sizes Unavailable: {0:?}")]
    TypeSizeUnavailable(String),
}

#[derive(Clone)]
pub struct EventsDecoder {
    metadata: Metadata,
    type_sizes: HashMap<String, usize>,
    // marker: PhantomData<fn() -> T>,
}

impl TryFrom<Metadata> for EventsDecoder {
    type Error = EventsError;

    fn try_from(metadata: Metadata) -> Result<Self, Self::Error> {
        let mut decoder = Self {
            metadata,
            type_sizes: HashMap::new(),
            // marker: PhantomData,
        };
        // register default event arg type sizes for dynamic decoding of events
        decoder.register_type_size::<bool>("bool")?;
        decoder.register_type_size::<u32>("ReferendumIndex")?;
        decoder.register_type_size::<[u8; 16]>("Kind")?;
        decoder.register_type_size::<[u8; 32]>("AuthorityId")?;
        decoder.register_type_size::<u8>("u8")?;
        decoder.register_type_size::<u32>("u32")?;
        decoder.register_type_size::<u64>("u64")?;
        decoder.register_type_size::<u32>("AccountIndex")?;
        decoder.register_type_size::<u32>("SessionIndex")?;
        decoder.register_type_size::<u32>("PropIndex")?;
        decoder.register_type_size::<u32>("ProposalIndex")?;
        decoder.register_type_size::<u32>("AuthorityIndex")?;
        decoder.register_type_size::<u64>("AuthorityWeight")?;
        decoder.register_type_size::<u32>("MemberCount")?;
        decoder.register_type_size::<AccountId>("AccountId")?;
        decoder.register_type_size::<BlockNumber>("BlockNumber")?;
        decoder.register_type_size::<Moment>("Moment")?;
        decoder.register_type_size::<Hash>("Hash")?;
        decoder.register_type_size::<Balance>("Balance")?;
        // VoteThreshold enum index
        decoder.register_type_size::<u8>("VoteThreshold")?;

        Ok(decoder)
    }
}

impl EventsDecoder {
    pub fn register_type_size<U>(&mut self, name: &str) -> Result<usize, EventsError>
    where
        U: Default + Codec + Send + 'static,
    {
        let size = U::default().encode().len();
        if size > 0 {
            self.type_sizes.insert(name.to_string(), size);
            Ok(size)
        } else {
            Err(EventsError::TypeSizeUnavailable(name.to_owned()))
        }
    }

    pub fn check_missing_type_sizes(&self) {
        let mut missing = HashSet::new();
        for module in self.metadata.modules_with_events() {
            for event in module.events() {
                for arg in event.arguments() {
                    for primitive in arg.primitives() {
                        if module.name() != "System"
                            && !self.type_sizes.contains_key(&primitive)
                            && !primitive.contains("PhantomData")
                        {
                            missing.insert(format!(
                                "{}::{}::{}",
                                module.name(),
                                event.name,
                                primitive
                            ));
                        }
                    }
                }
            }
        }
        if !missing.is_empty() {
            log::warn!(
                "The following primitive types do not have registered sizes: {:?} \
                If any of these events are received, an error will occur since we cannot decode them",
                missing
            );
        }
    }

    fn decode_raw_bytes<I: Input, W: Output>(
        &self,
        args: &[EventArg],
        input: &mut I,
        output: &mut W,
    ) -> Result<(), EventsError> {
        for arg in args {
            match arg {
                EventArg::Vec(arg) => {
                    let len = <Compact<u32>>::decode(input)?;
                    len.encode_to(output);
                    for _ in 0..len.0 {
                        self.decode_raw_bytes(&[*arg.clone()], input, output)?
                    }
                }
                EventArg::Tuple(args) => self.decode_raw_bytes(args, input, output)?,
                EventArg::Primitive(name) => {
                    if name.contains("PhantomData") {
                        // PhantomData is size 0
                        return Ok(());
                    }
                    if let Some(size) = self.type_sizes.get(name) {
                        let mut buf = vec![0; *size];
                        input.read(&mut buf)?;
                        output.write(&buf);
                    } else {
                        return Err(EventsError::TypeSizeUnavailable(name.to_owned()));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn decode_events(
        &self,
        input: &mut &[u8],
    ) -> Result<Vec<(Phase, RuntimeEvent)>, EventsError> {
        log::debug!("Decoding compact len: {:?}", input);
        let compact_len = <Compact<u32>>::decode(input)?;
        let len = compact_len.0 as usize;

        let mut r = Vec::new();
        for _ in 0..len {
            // decode EventRecord
            log::debug!("Decoding phase: {:?}", input);
            let phase = Phase::decode(input)?;
            let module_variant = input.read_byte()?;

            let module = self.metadata.module_with_events(module_variant)?;
            let event = if module.name() == "System" {
                log::debug!("Decoding system event, intput: {:?}", input);
                let system_event = SystemEvent::decode(input)?;
                log::debug!("Decoding successful, system_event: {:?}", system_event);
                RuntimeEvent::System(system_event)
            } else {
                let event_variant = input.read_byte()?;
                let event_metadata = module.event(event_variant)?;
                log::debug!(
                    "decoding event '{}::{}'",
                    module.name(),
                    event_metadata.name
                );

                let mut event_data = Vec::<u8>::new();
                self.decode_raw_bytes(&event_metadata.arguments(), input, &mut event_data)?;

                log::debug!(
                    "received event '{}::{}', raw bytes: {}",
                    module.name(),
                    event_metadata.name,
                    hex::encode(&event_data),
                );

                RuntimeEvent::Raw(RawEvent {
                    module: module.name().to_string(),
                    variant: event_metadata.name.clone(),
                    data: event_data,
                })
            };

            // topics come after the event data in EventRecord
            log::debug!("Phase {:?}, Event: {:?}", phase, event);

            log::debug!("Decoding topics {:?}", input);
            let _topics = Vec::<crate::Hash>::decode(input)?;
            r.push((phase, event));
        }
        Ok(r)
    }
}
