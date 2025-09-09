// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The transaction extension trait.

use alloc::vec::Vec;
use codec::{Codec, Decode, Encode};
use core::fmt::Debug;
#[doc(hidden)]
pub use core::marker::PhantomData;
use impl_trait_for_tuples::impl_for_tuples;
use scale_info::{MetaType, StaticTypeInfo};

use sp_runtime::traits::Dispatchable;

// mod as_transaction_extension;
// mod dispatch_transaction;
use crate::extrinsics::sp_runtime_copy::transaction_validity::TransactionValidityError;

/// Provides `Sealed` trait.
mod private {
	/// Special trait that prevents the implementation of some traits outside of this crate.
	pub trait Sealed {}
}

/// The base implication in a transaction.
///
/// This struct is used to represent the base implication in the transaction, that is
/// the implication not part of any transaction extensions. It usually comprises of the call and
/// the transaction extension version.
///
/// The concept of implication in the transaction extension pipeline is explained in the trait
/// documentation: [`TransactionExtension`].
#[derive(Encode)]
pub struct TxBaseImplication<T>(pub T);

impl<T: Encode> Implication for TxBaseImplication<T> {
	fn parts(&self) -> ImplicationParts<&impl Encode, &impl Encode, &impl Encode> {
		ImplicationParts { base: self, explicit: &(), implicit: &() }
	}
}

impl<T> private::Sealed for TxBaseImplication<T> {}

/// The implication in a transaction.
///
/// The concept of implication in the transaction extension pipeline is explained in the trait
/// documentation: [`TransactionExtension`].
#[derive(Encode)]
pub struct ImplicationParts<Base, Explicit, Implicit> {
	/// The base implication, that is implication not part of any transaction extension, usually
	/// the call and the transaction extension version.
	pub base: Base,
	/// The explicit implication in transaction extensions.
	pub explicit: Explicit,
	/// The implicit implication in transaction extensions.
	pub implicit: Implicit,
}

impl<Base: Encode, Explicit: Encode, Implicit: Encode> Implication
	for ImplicationParts<Base, Explicit, Implicit>
{
	fn parts(&self) -> ImplicationParts<&impl Encode, &impl Encode, &impl Encode> {
		ImplicationParts { base: &self.base, explicit: &self.explicit, implicit: &self.implicit }
	}
}

impl<Base, Explicit, Implicit> private::Sealed for ImplicationParts<Base, Explicit, Implicit> {}

/// Interface of implications in the transaction extension pipeline.
///
/// Implications can be encoded, this is useful for checking signature on the implications.
/// Implications can be split into parts, this allow to destructure and restructure the
/// implications, this is useful for nested pipeline.
///
/// This trait is sealed, consider using [`TxBaseImplication`] and [`ImplicationParts`]
/// implementations.
///
/// The concept of implication in the transaction extension pipeline is explained in the trait
/// documentation: [`TransactionExtension`].
pub trait Implication: Encode + private::Sealed {
	/// Destructure the implication into its parts.
	fn parts(&self) -> ImplicationParts<&impl Encode, &impl Encode, &impl Encode>;
}

/// Means by which a transaction may be extended. This type embodies both the data and the logic
/// that should be additionally associated with the transaction. It should be plain old data.
///
/// The simplest transaction extension would be the Unit type (and empty pipeline) `()`. This
/// executes no additional logic and implies a dispatch of the transaction's call using the
/// inherited origin (either `None` or `Signed`, depending on whether this is a signed or general
/// transaction).
///
/// Transaction extensions are capable of altering certain associated semantics:
///
/// - They may define the origin with which the transaction's call should be dispatched.
/// - They may define various parameters used by the transaction queue to determine under what
///   conditions the transaction should be retained and introduced on-chain.
/// - They may define whether this transaction is acceptable for introduction on-chain at all.
///
/// Each of these semantics are defined by the `validate` function.
///
/// **NOTE: Transaction extensions cannot under any circumstances alter the call itself.**
///
/// Transaction extensions are capable of defining logic which is executed additionally to the
/// dispatch of the call:
///
/// - They may define logic which must be executed prior to the dispatch of the call.
/// - They may also define logic which must be executed after the dispatch of the call.
///
/// Each of these semantics are defined by the `prepare` and `post_dispatch_details` functions
/// respectively.
///
/// Finally, transaction extensions may define additional data to help define the implications of
/// the logic they introduce. This additional data may be explicitly defined by the transaction
/// author (in which case it is included as part of the transaction body), or it may be implicitly
/// defined by the transaction extension based around the on-chain state (which the transaction
/// author is assumed to know). This data may be utilized by the above logic to alter how a node's
/// transaction queue treats this transaction.
///
/// ## Default implementations
///
/// Of the 6 functions in this trait along with `TransactionExtension`, 2 of them must return a
/// value of an associated type on success, with only `implicit` having a default implementation.
/// This means that default implementations cannot be provided for `validate` and `prepare`.
/// However, a macro is provided [impl_tx_ext_default](crate::impl_tx_ext_default) which is capable
/// of generating default implementations for both of these functions. If you do not wish to
/// introduce additional logic into the transaction pipeline, then it is recommended that you use
/// this macro to implement these functions. Additionally, [weight](TransactionExtension::weight)
/// can return a default value, which would mean the extension is weightless, but it is not
/// implemented by default. Instead, implementers can explicitly choose to implement this default
/// behavior through the same [impl_tx_ext_default](crate::impl_tx_ext_default) macro.
///
/// If your extension does any post-flight logic, then the functionality must be implemented in
/// [post_dispatch_details](TransactionExtension::post_dispatch_details). This function can return
/// the actual weight used by the extension during an entire dispatch cycle by wrapping said weight
/// value in a `Some`. This is useful in computing fee refunds, similar to how post dispatch
/// information is used to refund fees for calls. Alternatively, a `None` can be returned, which
/// means that the worst case scenario weight, namely the value returned by
/// [weight](TransactionExtension::weight), is the actual weight. This particular piece of logic
/// is embedded in the default implementation of
/// [post_dispatch](TransactionExtension::post_dispatch) so that the weight is assumed to be worst
/// case scenario, but implementers of this trait can correct it with extra effort. Therefore, all
/// users of an extension should use [post_dispatch](TransactionExtension::post_dispatch), with
/// [post_dispatch_details](TransactionExtension::post_dispatch_details) considered an internal
/// function.
///
/// ## Pipelines, Inherited Implications, and Authorized Origins
///
/// Requiring a single transaction extension to define all of the above semantics would be
/// cumbersome and would lead to a lot of boilerplate. Instead, transaction extensions are
/// aggregated into pipelines, which are tuples of transaction extensions. Each extension in the
/// pipeline is executed in order, and the output of each extension is aggregated and/or relayed as
/// the input to the next extension in the pipeline.
///
/// This ordered composition happens with all data types ([Val](TransactionExtension::Val),
/// [Pre](TransactionExtension::Pre) and [Implicit](TransactionExtension::Implicit)) as well as
/// all functions. There are important consequences stemming from how the composition affects the
/// meaning of the `origin` and `implication` parameters as well as the results. Whereas the
/// [prepare](TransactionExtension::prepare) and
/// [post_dispatch](TransactionExtension::post_dispatch) functions are clear in their meaning, the
/// [validate](TransactionExtension::validate) function is fairly sophisticated and warrants further
/// explanation.
///
/// Firstly, the `origin` parameter. The `origin` passed into the first item in a pipeline is simply
/// that passed into the tuple itself. It represents an authority who has authorized the implication
/// of the transaction, as of the extension it has been passed into *and any further extensions it
/// may pass though, all the way to, and including, the transaction's dispatch call itself. Each
/// following item in the pipeline is passed the origin which the previous item returned. The origin
/// returned from the final item in the pipeline is the origin which is returned by the tuple
/// itself.
///
/// This means that if a constituent extension returns a different origin to the one it was called
/// with, then (assuming no other extension changes it further) *this new origin will be used for
/// all extensions following it in the pipeline, and will be returned from the pipeline to be used
/// as the origin for the call's dispatch*. The call itself as well as all these extensions
/// following may each imply consequence for this origin. We call this the *inherited implication*.
///
/// The *inherited implication* is the cumulated on-chain effects born by whatever origin is
/// returned. It is expressed to the [validate](TransactionExtension::validate) function only as the
/// `implication` argument which implements the [Encode] trait. A transaction extension may define
/// its own implications through its own fields and the
/// [implicit](TransactionExtension::implicit) function. This is only utilized by extensions
/// which precede it in a pipeline or, if the transaction is an old-school signed transaction, the
/// underlying transaction verification logic.
///
/// **The inherited implication passed as the `implication` parameter to
/// [validate](TransactionExtension::validate) does not include the extension's inner data itself
/// nor does it include the result of the extension's `implicit` function.** If you both provide an
/// implication and rely on the implication, then you need to manually aggregate your extensions
/// implication with the aggregated implication passed in.
///
/// In the post dispatch pipeline, the actual weight of each extension is accrued in the
/// [PostDispatchInfo](PostDispatchInfoOf<Call>) of that transaction sequentially with each
/// [post_dispatch](TransactionExtension::post_dispatch) call. This means that an extension handling
/// transaction payment and refunds should be at the end of the pipeline in order to capture the
/// correct amount of weight used during the call. This is because one cannot know the actual weight
/// of an extension after post dispatch without running the post dispatch ahead of time.
pub trait TransactionExtension<Call>:
	Codec + Decode + Debug + Sync + Send + Clone + Eq + PartialEq + StaticTypeInfo
{
	/// Unique identifier of this signed extension.
	///
	/// This will be exposed in the metadata to identify the signed extension used in an extrinsic.
	const IDENTIFIER: &'static str;

	/// Any additional data which was known at the time of transaction construction and can be
	/// useful in authenticating the transaction. This is determined dynamically in part from the
	/// on-chain environment using the `implicit` function and not directly contained in the
	/// transaction itself and therefore is considered "implicit".
	type Implicit: Codec + StaticTypeInfo;

	/// Determine any additional data which was known at the time of transaction construction and
	/// can be useful in authenticating the transaction. The expected usage of this is to include in
	/// any data which is signed and verified as part of transaction validation. Also perform any
	/// pre-signature-verification checks and return an error if needed.
	fn implicit(&self) -> Result<Self::Implicit, TransactionValidityError> {
		use super::super::transaction_validity::InvalidTransaction::IndeterminateImplicit;
		Ok(Self::Implicit::decode(&mut &[][..]).map_err(|_| IndeterminateImplicit)?)
	}

	/// Returns the metadata for this extension.
	///
	/// As a [`TransactionExtension`] can be a tuple of [`TransactionExtension`]s we need to return
	/// a `Vec` that holds the metadata of each one. Each individual `TransactionExtension` must
	/// return *exactly* one [`TransactionExtensionMetadata`].
	///
	/// This method provides a default implementation that returns a vec containing a single
	/// [`TransactionExtensionMetadata`].
	fn metadata() -> Vec<TransactionExtensionMetadata> {
		alloc::vec![TransactionExtensionMetadata {
			identifier: Self::IDENTIFIER,
			ty: scale_info::meta_type::<Self>(),
			implicit: scale_info::meta_type::<Self::Implicit>()
		}]
	}

	/// The type that encodes information that can be passed from `validate` to `prepare`.
	type Val;

	/// The type that encodes information that can be passed from `prepare` to `post_dispatch`.
	type Pre;
}

/// Information about a [`TransactionExtension`] for the runtime metadata.
pub struct TransactionExtensionMetadata {
	/// The unique identifier of the [`TransactionExtension`].
	pub identifier: &'static str,
	/// The type of the [`TransactionExtension`].
	pub ty: MetaType,
	/// The type of the [`TransactionExtension`] additional signed data for the payload.
	pub implicit: MetaType,
}

#[impl_for_tuples(1, 12)]
impl<Call: Dispatchable> TransactionExtension<Call> for Tuple {
	const IDENTIFIER: &'static str = "Use `metadata()`!";
	for_tuples!( type Implicit = ( #( Tuple::Implicit ),* ); );
	fn implicit(&self) -> Result<Self::Implicit, TransactionValidityError> {
		Ok(for_tuples!( ( #( Tuple.implicit()? ),* ) ))
	}
	fn metadata() -> Vec<TransactionExtensionMetadata> {
		let mut ids = Vec::new();
		for_tuples!( #( ids.extend(Tuple::metadata()); )* );
		ids
	}

	for_tuples!( type Val = ( #( Tuple::Val ),* ); );
	for_tuples!( type Pre = ( #( Tuple::Pre ),* ); );
}

impl<Call: Dispatchable> TransactionExtension<Call> for () {
	const IDENTIFIER: &'static str = "UnitTransactionExtension";
	type Implicit = ();
	fn implicit(&self) -> core::result::Result<Self::Implicit, TransactionValidityError> {
		Ok(())
	}
	type Val = ();
	type Pre = ();
}
