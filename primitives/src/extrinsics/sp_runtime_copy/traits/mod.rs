use codec::{Decode, Encode};
use scale_info::TypeInfo;

pub mod transaction_extension;

pub use transaction_extension::TransactionExtension;

/// Something that acts like a [`SignaturePayload`](Extrinsic::SignaturePayload) of an
/// [`Extrinsic`].
pub trait SignaturePayload {
	/// The type of the address that signed the extrinsic.
	///
	/// Particular to a signed extrinsic.
	type SignatureAddress: TypeInfo;

	/// The signature type of the extrinsic.
	///
	/// Particular to a signed extrinsic.
	type Signature: TypeInfo;

	/// The additional data that is specific to the signed extrinsic.
	///
	/// Particular to a signed extrinsic.
	type SignatureExtra: TypeInfo;
}

impl SignaturePayload for () {
	type SignatureAddress = ();
	type Signature = ();
	type SignatureExtra = ();
}

/// Implementor is an [`Extrinsic`] and provides metadata about this extrinsic.
pub trait ExtrinsicMetadata {
	/// The format versions of the `Extrinsic`.
	///
	/// By format we mean the encoded representation of the `Extrinsic`.
	const VERSIONS: &'static [u8];

	/// Transaction extensions attached to this `Extrinsic`.
	type TransactionExtensions;
}

/// Something that acts like an `Extrinsic`.
pub trait ExtrinsicLike: Sized {
	/// Is this `Extrinsic` signed?
	/// If no information are available about signed/unsigned, `None` should be returned.
	#[deprecated = "Use and implement `!is_bare()` instead"]
	fn is_signed(&self) -> Option<bool> {
		None
	}

	/// Returns `true` if this `Extrinsic` is bare.
	fn is_bare(&self) -> bool {
		#[allow(deprecated)]
		!self.is_signed().unwrap_or(true)
	}
}

/// An extrinsic on which we can get access to call.
pub trait ExtrinsicCall: ExtrinsicLike {
	/// The type of the call.
	type Call;

	/// Get the call of the extrinsic.
	fn call(&self) -> &Self::Call;
}

/// Dispatchable impl containing an arbitrary value which panics if it actually is dispatched.
#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub struct FakeDispatchable<Inner>(pub Inner);
impl<Inner> From<Inner> for FakeDispatchable<Inner> {
	fn from(inner: Inner) -> Self {
		Self(inner)
	}
}
impl<Inner> FakeDispatchable<Inner> {
	/// Take `self` and return the underlying inner value.
	pub fn deconstruct(self) -> Inner {
		self.0
	}
}
impl<Inner> AsRef<Inner> for FakeDispatchable<Inner> {
	fn as_ref(&self) -> &Inner {
		&self.0
	}
}

// impl<Inner> Dispatchable for FakeDispatchable<Inner> {
// 	type RuntimeOrigin = ();
// 	type Config = ();
// 	type Info = ();
// 	type PostInfo = ();
// 	fn dispatch(
// 		self,
// 		_origin: Self::RuntimeOrigin,
// 	) -> crate::DispatchResultWithInfo<Self::PostInfo> {
// 		panic!("This implementation should not be used for actual dispatch.");
// 	}
// }
