use crate::metadata::{ErrorMetadata, EventMetadata, PalletMetadata};
use core::fmt::Formatter;
use sp_std::fmt::Display;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

impl Display for EventMetadata {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        writeln!(f, "Name: {:?}", self.event())?;
        writeln!(f, "Variant: {:?}", self.event())
    }
}

impl Display for ErrorMetadata {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        writeln!(f, "Name: {:?}", self.error())?;
        writeln!(f, "Description: {:?}", self.description())
    }
}

impl PalletMetadata {
    // pub fn format_header(&self) -> String {
    //     format!(
    //         "----------------- Pallet: '{}' -----------------\n\
    // 		Pallet id: {}",
    //         self.name, self.index
    //     )
    // }

    pub fn format_calls(&self) -> String {
        let mut string = format!(
            "----------------- Calls for Pallet: {} -----------------\n",
            self.name
        );

        for (name, index) in &self.calls {
            string.push_str(&format!("Name: {}, index {}", name, index));
        }
        string
    }

    pub fn format_constants(&self) -> String {
        let mut string = format!(
            "----------------- Constants for Pallet: {} -----------------\n",
            self.name
        );
        for (name, constant) in &self.constants {
            string.push_str(&format!(
                "Name: {}, Type {:?}, Value {:?}",
                name, constant.ty, constant.value
            ));
        }
        string
    }
    pub fn format_storages(&self) -> String {
        let mut string = format!(
            "----------------- Storages for Pallet: {} -----------------\n",
            self.name
        );
        for (name, storage) in &self.storage {
            #[cfg(feature = "std")]
            string.push_str(&format!(
                "Name: {}, Modifier: {:?}, Type {:?}, Default {:?}",
                name, storage.modifier, storage.ty, storage.default
            ));

            #[cfg(not(feature = "std"))]
            string.push_str(&format!("Name: {}, Default {:?}", name, storage.default,));
        }
        string
    }
}
