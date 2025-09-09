use scale_info::TypeInfo;

pub mod transaction_extension;

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
