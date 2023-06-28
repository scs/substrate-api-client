/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG
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

use crate::metadata::{Metadata, PalletMetadata};

impl Metadata {
	pub fn print_overview(&self) {
		let mut string = String::new();
		for pallet in self.pallets() {
			string.push_str(pallet.name());
			string.push('\n');
			for storage in pallet.storage() {
				string.push_str(" s  ");
				string.push_str(&storage.name);
				string.push('\n');
			}

			if let Some(call_variants) = pallet.call_variants() {
				for call in call_variants {
					string.push_str(" c  ");
					string.push_str(&call.name);
					string.push('\n');
				}
			}

			for constant in pallet.constants() {
				string.push_str(" cst  ");
				string.push_str(&constant.name);
				string.push('\n');
			}

			if let Some(events) = pallet.event_variants() {
				for event in events {
					string.push_str(" e  ");
					string.push_str(&event.name);
					string.push('\n');
				}
			}

			if let Some(errors) = pallet.error_variants() {
				for error in errors {
					string.push_str(" err  ");
					string.push_str(&error.name);
					string.push('\n');
				}
			}
		}

		println!("{string}");
	}

	pub fn print_pallets(&self) {
		for pallet in self.pallets() {
			pallet.print()
		}
	}

	pub fn print_pallets_with_calls(&self) {
		for pallet in self.pallets() {
			pallet.print_calls();
		}
	}
	pub fn print_pallets_with_constants(&self) {
		for pallet in self.pallets() {
			pallet.print_constants();
		}
	}
	pub fn print_pallet_with_storages(&self) {
		for pallet in self.pallets() {
			pallet.print_storages();
		}
	}

	pub fn print_pallets_with_events(&self) {
		for pallet in self.pallets() {
			pallet.print_events();
		}
	}

	pub fn print_pallets_with_errors(&self) {
		for pallet in self.pallets() {
			pallet.print_errors();
		}
	}
}

impl<'a> PalletMetadata<'a> {
	pub fn print(&self) {
		println!("----------------- Pallet: '{}' -----------------\n", self.name());
		println!("Pallet id: {}", self.index());
	}

	pub fn print_calls(&self) {
		println!("----------------- Calls for Pallet: {} -----------------\n", self.name());
		if let Some(variants) = self.call_variants() {
			for variant in variants {
				println!("Name: {}, index {}", variant.name, variant.index);
			}
		};
		println!();
	}

	pub fn print_constants(&self) {
		println!("----------------- Constants for Pallet: {} -----------------\n", self.name());
		for constant in self.constants() {
			println!("Name: {}, Type {:?}, Value {:?}", constant.name, constant.ty, constant.value);
		}
		println!();
	}
	pub fn print_storages(&self) {
		println!("----------------- Storages for Pallet: {} -----------------\n", self.name());
		for storage in self.storage() {
			println!(
				"Name: {}, Modifier: {:?}, Type {:?}, Default {:?}",
				storage.name, storage.modifier, storage.ty, storage.default
			);
		}
		println!();
	}

	pub fn print_events(&self) {
		println!("----------------- Events for Pallet: {} -----------------\n", self.name());
		if let Some(variants) = self.event_variants() {
			for variant in variants {
				println!("Name: {}", variant.name);
				println!("Field: {:?}", variant.fields);
				println!("Docs: {:?}", variant.docs);
				println!();
			}
		};
		println!();
	}

	pub fn print_errors(&self) {
		println!("----------------- Errors for Pallet: {} -----------------\n", self.name());
		if let Some(variants) = self.error_variants() {
			for variant in variants {
				println!("Name: {}", variant.name);
				println!("Docs: {:?}", variant.docs);
				println!();
			}
		};
		println!();
	}
}
