pub use crate::SubstrateOpaqueExtrinsic as OpaqueExtrinsic;

pub mod extrinsic;
pub mod traits;
pub mod transaction_validity;

pub mod generic {
	pub use super::extrinsic::*;
}

impl traits::ExtrinsicLike for OpaqueExtrinsic {
	fn is_bare(&self) -> bool {
		false
	}
}
