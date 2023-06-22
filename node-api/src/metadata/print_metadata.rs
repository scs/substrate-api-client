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

use crate::metadata::{ErrorMetadata, EventMetadata, Metadata, PalletMetadata};

impl Metadata {
	pub fn print_overview(&self) {
		let mut string = String::new();
		for (name, pallet) in &self.pallets {
			string.push_str(name.as_str());
			string.push('\n');
			for storage in pallet.storage.keys() {
				string.push_str(" s  ");
				string.push_str(storage.as_str());
				string.push('\n');
			}

			for call in pallet.call_indexes.keys() {
				string.push_str(" c  ");
				string.push_str(call.as_str());
				string.push('\n');
			}

			for constant in pallet.constants.keys() {
				string.push_str(" cst  ");
				string.push_str(constant.as_str());
				string.push('\n');
			}

			for event in self.events(pallet.index) {
				string.push_str(" e  ");
				string.push_str(event.event());
				string.push('\n');
			}

			for error in self.errors(pallet.index) {
				string.push_str(" err  ");
				string.push_str(error.error());
				string.push('\n');
			}
		}

		println!("{string}");
	}

	pub fn print_pallets(&self) {
		for m in self.pallets.values() {
			m.print()
		}
	}

	pub fn print_pallets_with_calls(&self) {
		for m in self.pallets.values() {
			if !m.call_indexes.is_empty() {
				m.print_calls();
			}
		}
	}
	pub fn print_pallets_with_constants(&self) {
		for m in self.pallets.values() {
			if !m.constants.is_empty() {
				m.print_constants();
			}
		}
	}
	pub fn print_pallet_with_storages(&self) {
		for m in self.pallets.values() {
			if !m.storage.is_empty() {
				m.print_storages();
			}
		}
	}

	pub fn print_pallets_with_events(&self) {
		for pallet in self.pallets.values() {
			println!("----------------- Events for Pallet: {} -----------------\n", pallet.name);
			for m in self.events(pallet.index) {
				m.print();
			}
			println!();
		}
	}

	pub fn print_pallets_with_errors(&self) {
		for pallet in self.pallets.values() {
			println!("----------------- Errors for Pallet: {} -----------------\n", pallet.name);
			for m in self.errors(pallet.index) {
				m.print();
			}
			println!();
		}
	}
}

impl PalletMetadata {
	pub fn print(&self) {
		println!("----------------- Pallet: '{}' -----------------\n", self.name);
		println!("Pallet id: {}", self.index);

		//self.print_calls();
	}

	pub fn print_calls(&self) {
		println!("----------------- Calls for Pallet: {} -----------------\n", self.name);
		for (name, index) in &self.call_indexes {
			println!("Name: {name}, index {index}");
		}
		println!();
	}

	pub fn print_constants(&self) {
		println!("----------------- Constants for Pallet: {} -----------------\n", self.name);
		for (name, constant) in &self.constants {
			println!("Name: {}, Type {:?}, Value {:?}", name, constant.ty, constant.value);
		}
		println!();
	}
	pub fn print_storages(&self) {
		println!("----------------- Storages for Pallet: {} -----------------\n", self.name);
		for (name, storage) in &self.storage {
			println!(
				"Name: {}, Modifier: {:?}, Type {:?}, Default {:?}",
				name, storage.modifier, storage.ty, storage.default
			);
		}
		println!();
	}
}

impl EventMetadata {
	pub fn print(&self) {
		println!("Name: {}", self.event());
		println!("Field: {:?}", self.fields());
		println!("Docs: {:?}", self.docs());
		println!()
	}
}

impl ErrorMetadata {
	pub fn print(&self) {
		println!("Name: {}", self.error());
		println!("Docs: {:?}", self.docs());
		println!()
	}
}
