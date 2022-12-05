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
use codec::MaxEncodedLen;
use scale_info::TypeInfo;
use sp_runtime::traits::{AtLeast32BitUnsigned, Get, MaybeSerializeDeserialize, Member};

/// Simplifed pallet assets Config trait. Needed because substrate pallets compile to wasm
/// in no_std mode.
pub trait AssetsConfig: crate::FrameSystemConfig {
	type RuntimeEvent;
	type Balance: AtLeast32BitUnsigned
		+ Default
		+ Copy
		+ MaybeSerializeDeserialize
		+ MaxEncodedLen
		+ TypeInfo;
	type RemoveItemsLimit: Get<u32>;
	type AssetId: Member + Copy + MaybeSerializeDeserialize + MaxEncodedLen;
	type AssetIdParameter: Copy + From<Self::AssetId> + Into<Self::AssetId> + MaxEncodedLen;
	type Currency;
	type CreateOrigin;
	type ForceOrigin;
	type AssetDeposit;
	type AssetAccountDeposit;
	type MetadataDepositBase;
	type MetadataDepositPerByte;
	type ApprovalDeposit;
	type StringLimit: Get<u32>;
	type Freezer;
	type Extra: Member + Default + MaxEncodedLen;
	type WeightInfo;
}
