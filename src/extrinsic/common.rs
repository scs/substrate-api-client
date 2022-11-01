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

// Common types.

use codec::{Decode, Encode};
use node_template_runtime::RuntimeCall;
use sp_runtime::AccountId32;
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
pub struct PayoutStakers {
    pub validator_stash: AccountId32,
    pub era: u32,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug)]
pub struct BatchPayout {
    pub calls: Vec<([u8; 2], PayoutStakers)>,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug)]
pub struct ForceEra {}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug)]
pub struct Batch {
    pub calls: Vec<RuntimeCall>,
}
