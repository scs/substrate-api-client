// This file was taken from scale-decode (Parity Technologies (UK))
// https://github.com/paritytech/scale-decode/
// And was adapted by Supercomputing Systems AG.
//
// Copyright (C) 2022-2023 Parity Technologies (UK) Ltd. (admin@parity.io)
// This file is a part of the scale-decode crate.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//         http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "primitive-types")]
mod primitive_types;

use crate::scale_decode::{
	error::{Error, ErrorKind},
	visitor::{
		self, decode_with_visitor, types::*, DecodeAsTypeResult, DecodeItemIterator, TypeId,
		Visitor,
	},
	DecodeAsFields, FieldIter, IntoVisitor,
};
use alloc::{
	borrow::{Cow, ToOwned},
	boxed::Box,
	string::{String, ToString},
	vec,
	vec::Vec,
};
use codec::Compact;
use core::num::{
	NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
	NonZeroU32, NonZeroU64, NonZeroU8,
};
use scale_bits::Bits;

use alloc::{
	collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
	rc::Rc,
	sync::Arc,
};
use core::{
	marker::PhantomData,
	ops::{Range, RangeInclusive},
	time::Duration,
};

pub struct BasicVisitor<T> {
	_marker: core::marker::PhantomData<T>,
}

/// Generate an [`IntoVisitor`] impl for basic types `T` where `BasicVisitor<T>` impls `Visitor`.
macro_rules! impl_into_visitor {
    ($ty:ident $(< $($lt:lifetime,)* $($param:ident),* >)? $(where $( $where:tt )* )?) => {
        impl $(< $($lt,)* $($param),* >)? crate::scale_decode::IntoVisitor for $ty $(< $($lt,)* $($param),* >)?
        where
            BasicVisitor<$ty $(< $($lt,)* $($param),* >)?>: for<'scale, 'info> Visitor<Error = Error, Value<'scale, 'info> = Self>,
            $( $($where)* )?
        {
            type Visitor = BasicVisitor<$ty $(< $($lt,)* $($param),* >)?>;
            fn into_visitor() -> Self::Visitor {
                BasicVisitor { _marker: core::marker::PhantomData }
            }
        }
    };
}

/// Ignore single-field tuples/composites and visit the single field inside instead.
macro_rules! visit_single_field_composite_tuple_impls {
	() => {
		fn visit_composite<'scale, 'info>(
			self,
			value: &mut $crate::scale_decode::visitor::types::Composite<'scale, 'info>,
			_type_id: $crate::scale_decode::visitor::TypeId,
		) -> Result<Self::Value<'scale, 'info>, Self::Error> {
			if value.remaining() != 1 {
				return self.visit_unexpected($crate::scale_decode::visitor::Unexpected::Composite)
			}
			value.decode_item(self).unwrap()
		}
		fn visit_tuple<'scale, 'info>(
			self,
			value: &mut $crate::scale_decode::visitor::types::Tuple<'scale, 'info>,
			_type_id: $crate::scale_decode::visitor::TypeId,
		) -> Result<Self::Value<'scale, 'info>, Self::Error> {
			if value.remaining() != 1 {
				return self.visit_unexpected($crate::scale_decode::visitor::Unexpected::Tuple)
			}
			value.decode_item(self).unwrap()
		}
	};
}

impl Visitor for BasicVisitor<char> {
	type Error = Error;
	type Value<'scale, 'info> = char;

	fn visit_char<'scale, 'info>(
		self,
		value: char,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		Ok(value)
	}
	visit_single_field_composite_tuple_impls!();
}
impl_into_visitor!(char);

impl Visitor for BasicVisitor<bool> {
	type Error = Error;
	type Value<'scale, 'info> = bool;

	fn visit_bool<'scale, 'info>(
		self,
		value: bool,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		Ok(value)
	}
	visit_single_field_composite_tuple_impls!();
}
impl_into_visitor!(bool);

impl Visitor for BasicVisitor<String> {
	type Error = Error;
	type Value<'scale, 'info> = String;

	fn visit_str<'scale, 'info>(
		self,
		value: &mut Str<'scale>,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		let s = value.as_str()?.to_owned();
		Ok(s)
	}
	visit_single_field_composite_tuple_impls!();
}
impl_into_visitor!(String);

impl Visitor for BasicVisitor<Bits> {
	type Error = Error;
	type Value<'scale, 'info> = Bits;

	fn visit_bitsequence<'scale, 'info>(
		self,
		value: &mut BitSequence<'scale>,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		value
			.decode()?
			.collect::<Result<Bits, _>>()
			.map_err(|e| Error::new(ErrorKind::VisitorDecodeError(e.into())))
	}
	visit_single_field_composite_tuple_impls!();
}
impl_into_visitor!(Bits);

impl<T> Visitor for BasicVisitor<PhantomData<T>> {
	type Error = Error;
	type Value<'scale, 'info> = PhantomData<T>;

	fn visit_tuple<'scale, 'info>(
		self,
		value: &mut Tuple<'scale, 'info>,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		if value.remaining() == 0 {
			Ok(PhantomData)
		} else {
			self.visit_unexpected(visitor::Unexpected::Tuple)
		}
	}
	fn visit_composite<'scale, 'info>(
		self,
		value: &mut Composite<'scale, 'info>,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		if value.remaining() == 0 {
			Ok(PhantomData)
		} else {
			self.visit_unexpected(visitor::Unexpected::Composite)
		}
	}
}
impl_into_visitor!(PhantomData<T>);

// Generate impls to encode things based on some other type. We do this by implementing
// `IntoVisitor` and using the `AndThen` combinator to map from an existing one to the desired output.
macro_rules! impl_into_visitor_like {
    ($target:ident $(< $($lt:lifetime,)* $($param:ident),* >)? as $source:ty $( [where $($where:tt)*] )?: $mapper:expr) => {
        impl $(< $($lt,)* $($param),* >)? Visitor for BasicVisitor<$target $(< $($lt,)* $($param),* >)?>
        where
            $source: IntoVisitor,
            $( $($where)* )?
        {
            type Value<'scale, 'info> = $target $(< $($lt,)* $($param),* >)?;
            type Error = <<$source as IntoVisitor>::Visitor as Visitor>::Error;

            fn unchecked_decode_as_type<'scale, 'info>(
                self,
                input: &mut &'scale [u8],
                type_id: TypeId,
                types: &'info scale_info::PortableRegistry,
            ) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'info>, Self::Error>> {
                // Use the source visitor to decode into some type:
                let inner_res = decode_with_visitor(input, type_id.0, types, <$source>::into_visitor());
                // map this type into our desired output and return it:
                let res = inner_res.map($mapper);
                DecodeAsTypeResult::Decoded(res)
            }
        }
        impl_into_visitor!($target $(< $($lt,)* $($param),* >)?);
    }
}

impl_into_visitor_like!(Compact<T> as T: |res| Compact(res));
impl_into_visitor_like!(Arc<T> as T: |res| Arc::new(res));
impl_into_visitor_like!(Rc<T> as T: |res| Rc::new(res));
impl_into_visitor_like!(Box<T> as T: |res| Box::new(res));
impl_into_visitor_like!(Duration as (u64, u32): |res: (u64,u32)| Duration::from_secs(res.0) + Duration::from_nanos(res.1 as u64));
impl_into_visitor_like!(Range<T> as (T, T): |res: (T,T)| res.0..res.1);
impl_into_visitor_like!(RangeInclusive<T> as (T, T): |res: (T,T)| res.0..=res.1);

// A custom implementation for `Cow` because it's rather tricky; the visitor we want is whatever the
// `ToOwned` value for the Cow is, and Cow's have specific constraints, too.
impl<'a, T> Visitor for BasicVisitor<Cow<'a, T>>
where
	T: 'a + ToOwned + ?Sized,
	<T as ToOwned>::Owned: IntoVisitor,
{
	type Value<'scale, 'info> = Cow<'a, T>;
	type Error = <<<T as ToOwned>::Owned as IntoVisitor>::Visitor as Visitor>::Error;

	fn unchecked_decode_as_type<'scale, 'info>(
		self,
		input: &mut &'scale [u8],
		type_id: TypeId,
		types: &'info scale_info::PortableRegistry,
	) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'info>, Self::Error>> {
		// Use the ToOwned visitor to decode into some type:
		let inner_res =
			decode_with_visitor(input, type_id.0, types, <<T as ToOwned>::Owned>::into_visitor());
		// map this type into our owned Cow to return:
		let res = inner_res.map(Cow::Owned);
		DecodeAsTypeResult::Decoded(res)
	}
}
impl<'a, T> IntoVisitor for Cow<'a, T>
where
	T: 'a + ToOwned + ?Sized,
	<T as ToOwned>::Owned: IntoVisitor,
{
	type Visitor = BasicVisitor<Cow<'a, T>>;
	fn into_visitor() -> Self::Visitor {
		BasicVisitor { _marker: core::marker::PhantomData }
	}
}

macro_rules! impl_decode_seq_via_collect {
    ($ty:ident<$generic:ident> $(where $($where:tt)*)?) => {
        impl <$generic> Visitor for BasicVisitor<$ty<$generic>>
        where
            $generic: IntoVisitor,
            Error: From<<$generic::Visitor as Visitor>::Error>,
            $( $($where)* )?
        {
            type Value<'scale, 'info> = $ty<$generic>;
            type Error = Error;

            fn visit_sequence<'scale, 'info>(
                self,
                value: &mut Sequence<'scale, 'info>,
                _type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                decode_items_using::<_, $generic>(value).collect()
            }
            fn visit_array<'scale, 'info>(
                self,
                value: &mut Array<'scale, 'info>,
                _type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                decode_items_using::<_, $generic>(value).collect()
            }

            visit_single_field_composite_tuple_impls!();
        }
        impl_into_visitor!($ty < $generic > $( where $($where)* )?);
    }
}
impl_decode_seq_via_collect!(Vec<T>);
impl_decode_seq_via_collect!(VecDeque<T>);
impl_decode_seq_via_collect!(LinkedList<T>);
impl_decode_seq_via_collect!(BinaryHeap<T> where T: Ord);
impl_decode_seq_via_collect!(BTreeSet<T> where T: Ord);

// For arrays of fixed lengths, we decode to a vec first and then try to turn that into the fixed size array.
// Like vecs, we can decode from tuples, sequences or arrays if the types line up ok.
macro_rules! array_method_impl {
	($value:ident, [$t:ident; $n:ident]) => {{
		let val = decode_items_using::<_, $t>($value).collect::<Result<Vec<$t>, _>>()?;
		let actual_len = val.len();
		let arr = val
			.try_into()
			.map_err(|_e| Error::new(ErrorKind::WrongLength { actual_len, expected_len: $n }))?;
		Ok(arr)
	}};
}
impl<const N: usize, T> Visitor for BasicVisitor<[T; N]>
where
	T: IntoVisitor,
	Error: From<<T::Visitor as Visitor>::Error>,
{
	type Value<'scale, 'info> = [T; N];
	type Error = Error;

	fn visit_sequence<'scale, 'info>(
		self,
		value: &mut Sequence<'scale, 'info>,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		array_method_impl!(value, [T; N])
	}
	fn visit_array<'scale, 'info>(
		self,
		value: &mut Array<'scale, 'info>,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		array_method_impl!(value, [T; N])
	}

	visit_single_field_composite_tuple_impls!();
}
impl<const N: usize, T> IntoVisitor for [T; N]
where
	T: IntoVisitor,
	Error: From<<T::Visitor as Visitor>::Error>,
{
	type Visitor = BasicVisitor<[T; N]>;
	fn into_visitor() -> Self::Visitor {
		BasicVisitor { _marker: core::marker::PhantomData }
	}
}

impl<T> Visitor for BasicVisitor<BTreeMap<String, T>>
where
	T: IntoVisitor,
	Error: From<<T::Visitor as Visitor>::Error>,
{
	type Error = Error;
	type Value<'scale, 'info> = BTreeMap<String, T>;

	fn visit_composite<'scale, 'info>(
		self,
		value: &mut Composite<'scale, 'info>,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		let mut map = BTreeMap::new();
		while value.remaining() > 0 {
			// Get the name. If no name, skip over the corresponding value.
			let Some(key) = value.peek_name() else {
                value.decode_item(crate::scale_decode::visitor::IgnoreVisitor).transpose()?;
                continue;
            };
			// Decode the value now that we have a valid name.
			let Some(val) = value.decode_item(T::into_visitor()) else {
                break
            };
			// Save to the map.
			let val = val.map_err(|e| Error::from(e).at_field(key.to_owned()))?;
			map.insert(key.to_owned(), val);
		}
		Ok(map)
	}
}
impl_into_visitor!(BTreeMap<String, T>);

impl<T> Visitor for BasicVisitor<Option<T>>
where
	T: IntoVisitor,
	Error: From<<T::Visitor as Visitor>::Error>,
{
	type Error = Error;
	type Value<'scale, 'info> = Option<T>;

	fn visit_variant<'scale, 'info>(
		self,
		value: &mut Variant<'scale, 'info>,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		if value.name() == "Some" && value.fields().remaining() == 1 {
			let val = value
				.fields()
				.decode_item(T::into_visitor())
				.transpose()
				.map_err(|e| Error::from(e).at_variant("Some"))?
				.expect("checked for 1 field already so should be ok");
			Ok(Some(val))
		} else if value.name() == "None" && value.fields().remaining() == 0 {
			Ok(None)
		} else {
			Err(Error::new(ErrorKind::CannotFindVariant {
				got: value.name().to_string(),
				expected: vec!["Some", "None"],
			}))
		}
	}
	visit_single_field_composite_tuple_impls!();
}
impl_into_visitor!(Option<T>);

impl<T, E> Visitor for BasicVisitor<Result<T, E>>
where
	T: IntoVisitor,
	Error: From<<T::Visitor as Visitor>::Error>,
	E: IntoVisitor,
	Error: From<<E::Visitor as Visitor>::Error>,
{
	type Error = Error;
	type Value<'scale, 'info> = Result<T, E>;

	fn visit_variant<'scale, 'info>(
		self,
		value: &mut Variant<'scale, 'info>,
		_type_id: visitor::TypeId,
	) -> Result<Self::Value<'scale, 'info>, Self::Error> {
		if value.name() == "Ok" && value.fields().remaining() == 1 {
			let val = value
				.fields()
				.decode_item(T::into_visitor())
				.transpose()
				.map_err(|e| Error::from(e).at_variant("Ok"))?
				.expect("checked for 1 field already so should be ok");
			Ok(Ok(val))
		} else if value.name() == "Err" && value.fields().remaining() == 1 {
			let val = value
				.fields()
				.decode_item(E::into_visitor())
				.transpose()
				.map_err(|e| Error::from(e).at_variant("Err"))?
				.expect("checked for 1 field already so should be ok");
			Ok(Err(val))
		} else {
			Err(Error::new(ErrorKind::CannotFindVariant {
				got: value.name().to_string(),
				expected: vec!["Ok", "Err"],
			}))
		}
	}
	visit_single_field_composite_tuple_impls!();
}
impl_into_visitor!(Result<T, E>);

// Impl Visitor/DecodeAsType for all primitive number types
macro_rules! visit_number_fn_impl {
	($name:ident : $ty:ty where |$res:ident| $expr:expr) => {
		fn $name<'scale, 'info>(
			self,
			value: $ty,
			_type_id: visitor::TypeId,
		) -> Result<Self::Value<'scale, 'info>, Self::Error> {
			let $res = value;
			let n = $expr.ok_or_else(|| {
				Error::new(ErrorKind::NumberOutOfRange { value: value.to_string() })
			})?;
			Ok(n)
		}
	};
}
macro_rules! visit_number_impl {
    ($ty:ident where |$res:ident| $expr:expr) => {
        #[allow(clippy::useless_conversion)]
        impl Visitor for BasicVisitor<$ty> {
            type Error = Error;
            type Value<'scale, 'info> = $ty;

            visit_number_fn_impl!(visit_u8: u8 where |$res| $expr);
            visit_number_fn_impl!(visit_u16: u16 where |$res| $expr);
            visit_number_fn_impl!(visit_u32: u32 where |$res| $expr);
            visit_number_fn_impl!(visit_u64: u64 where |$res| $expr);
            visit_number_fn_impl!(visit_u128: u128 where |$res| $expr);
            visit_number_fn_impl!(visit_i8: i8 where |$res| $expr);
            visit_number_fn_impl!(visit_i16: i16 where |$res| $expr);
            visit_number_fn_impl!(visit_i32: i32 where |$res| $expr);
            visit_number_fn_impl!(visit_i64: i64 where |$res| $expr);
            visit_number_fn_impl!(visit_i128: i128 where |$res| $expr);

            visit_single_field_composite_tuple_impls!();
        }
        impl_into_visitor!($ty);
    };
}
visit_number_impl!(u8 where |res| res.try_into().ok());
visit_number_impl!(u16 where |res| res.try_into().ok());
visit_number_impl!(u32 where |res| res.try_into().ok());
visit_number_impl!(u64 where |res| res.try_into().ok());
visit_number_impl!(u128 where |res| res.try_into().ok());
visit_number_impl!(usize where |res| res.try_into().ok());
visit_number_impl!(i8 where |res| res.try_into().ok());
visit_number_impl!(i16 where |res| res.try_into().ok());
visit_number_impl!(i32 where |res| res.try_into().ok());
visit_number_impl!(i64 where |res| res.try_into().ok());
visit_number_impl!(i128 where |res| res.try_into().ok());
visit_number_impl!(isize where |res| res.try_into().ok());
visit_number_impl!(NonZeroU8 where |res| res.try_into().ok().and_then(NonZeroU8::new));
visit_number_impl!(NonZeroU16 where |res| res.try_into().ok().and_then(NonZeroU16::new));
visit_number_impl!(NonZeroU32 where |res| res.try_into().ok().and_then(NonZeroU32::new));
visit_number_impl!(NonZeroU64 where |res| res.try_into().ok().and_then(NonZeroU64::new));
visit_number_impl!(NonZeroU128 where |res| res.try_into().ok().and_then(NonZeroU128::new));
visit_number_impl!(NonZeroI8 where |res| res.try_into().ok().and_then(NonZeroI8::new));
visit_number_impl!(NonZeroI16 where |res| res.try_into().ok().and_then(NonZeroI16::new));
visit_number_impl!(NonZeroI32 where |res| res.try_into().ok().and_then(NonZeroI32::new));
visit_number_impl!(NonZeroI64 where |res| res.try_into().ok().and_then(NonZeroI64::new));
visit_number_impl!(NonZeroI128 where |res| res.try_into().ok().and_then(NonZeroI128::new));

macro_rules! count_idents {
    ($t:ident $($rest:ident)*) => {
        1 + count_idents!( $($rest)* )
    };
    () => {
        0
    }
}

// Decode tuple types from any matching type.
macro_rules! tuple_method_impl {
    (($($t:ident,)*), $value:ident) => {{
        const EXPECTED_LEN: usize = count_idents!($($t)*);
        if $value.remaining() != EXPECTED_LEN {
            return Err(Error::new(ErrorKind::WrongLength {
                actual_len: $value.remaining(),
                expected_len: EXPECTED_LEN
            }))
        }

        #[allow(unused)]
        let mut idx = 0;

        Ok((
            $(
                #[allow(unused_assignments)]
                {
                    let v = $value
                        .decode_item($t::into_visitor())
                        .transpose()
                        .map_err(|e| Error::from(e).at_idx(idx))?
                        .expect("length already checked via .remaining()");
                    idx += 1;
                    v
                }
            ,)*
        ))
    }}
}
macro_rules! decode_inner_type_when_one_tuple_entry {
	($t:ident) => {
		fn unchecked_decode_as_type<'scale, 'info>(
			self,
			input: &mut &'scale [u8],
			type_id: TypeId,
			types: &'info scale_info::PortableRegistry,
		) -> DecodeAsTypeResult<Self, Result<Self::Value<'scale, 'info>, Self::Error>> {
			// [jsdw]: See https://github.com/rust-lang/rustfmt/issues/5062.
			// let else formatting not stable in macros; will keep indenting!
			#[rustfmt::skip]
            let Some(ty) = types.resolve(type_id.0) else {
                return DecodeAsTypeResult::Skipped(self);
            };

			// Get the inner type ID if the thing we're trying to decode isn't
			// a tuple or composite value. Else, fall back to default behaviour.
			// This ensures that this function strictly improves on the default
			// which would be to fail.
			let inner_type_id = match &ty.type_def {
				scale_info::TypeDef::Composite(_) => return DecodeAsTypeResult::Skipped(self),
				scale_info::TypeDef::Tuple(_) => return DecodeAsTypeResult::Skipped(self),
				_ => type_id.0,
			};

			let inner_res = decode_with_visitor(input, inner_type_id, types, <$t>::into_visitor());
			let res = inner_res.map(|val| (val,)).map_err(|e| e.into());
			DecodeAsTypeResult::Decoded(res)
		}
	};
	($($tt:tt)*) => {
		/* nothing */
	};
}
macro_rules! impl_decode_tuple {
    ($($t:ident)*) => {
        impl < $($t),* > Visitor for BasicVisitor<($($t,)*)>
        where $(
            $t: IntoVisitor,
            Error: From<<$t::Visitor as Visitor>::Error>,
        )*
        {
            type Value<'scale, 'info> = ($($t,)*);
            type Error = Error;

            // If we're trying to decode to a 1-tuple, and the type we're decoding
            // isn't a tuple or composite, then decode thye inner type and add the tuple.
            decode_inner_type_when_one_tuple_entry!($($t)*);

            fn visit_composite<'scale, 'info>(
                self,
                value: &mut Composite<'scale, 'info>,
                _type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                tuple_method_impl!(($($t,)*), value)
            }
            fn visit_tuple<'scale, 'info>(
                self,
                value: &mut Tuple<'scale, 'info>,
                _type_id: visitor::TypeId,
            ) -> Result<Self::Value<'scale, 'info>, Self::Error> {
                tuple_method_impl!(($($t,)*), value)
            }
        }

        // We can turn this tuple into a visitor which knows how to decode it:
        impl < $($t),* > IntoVisitor for ($($t,)*)
        where $( $t: IntoVisitor, Error: From<<$t::Visitor as Visitor>::Error>, )*
        {
            type Visitor = BasicVisitor<($($t,)*)>;
            fn into_visitor() -> Self::Visitor {
                BasicVisitor { _marker: core::marker::PhantomData }
            }
        }

        // We can decode given a list of fields (just delegate to the visitor impl:
        impl < $($t),* > DecodeAsFields for ($($t,)*)
        where $( $t: IntoVisitor, Error: From<<$t::Visitor as Visitor>::Error>, )*
        {
            fn decode_as_fields<'info>(input: &mut &[u8], fields: &mut dyn FieldIter<'info>, types: &'info scale_info::PortableRegistry) -> Result<Self, Error> {
                let mut composite = crate::scale_decode::visitor::types::Composite::new(input, crate::scale_decode::EMPTY_SCALE_INFO_PATH, fields, types);
                let val = <($($t,)*)>::into_visitor().visit_composite(&mut composite, crate::scale_decode::visitor::TypeId(0));

                // Skip over bytes that we decoded:
                composite.skip_decoding()?;
                *input = composite.bytes_from_undecoded();

                val
            }
        }
    }
}

impl_decode_tuple!();
impl_decode_tuple!(A);
impl_decode_tuple!(A B);
impl_decode_tuple!(A B C);
impl_decode_tuple!(A B C D);
impl_decode_tuple!(A B C D E);
impl_decode_tuple!(A B C D E F);
impl_decode_tuple!(A B C D E F G);
impl_decode_tuple!(A B C D E F G H);
impl_decode_tuple!(A B C D E F G H I);
impl_decode_tuple!(A B C D E F G H I J);
impl_decode_tuple!(A B C D E F G H I J K);
impl_decode_tuple!(A B C D E F G H I J K L);
impl_decode_tuple!(A B C D E F G H I J K L M);
impl_decode_tuple!(A B C D E F G H I J K L M N);
impl_decode_tuple!(A B C D E F G H I J K L M N O);
impl_decode_tuple!(A B C D E F G H I J K L M N O P);
impl_decode_tuple!(A B C D E F G H I J K L M N O P Q);
impl_decode_tuple!(A B C D E F G H I J K L M N O P Q R);
impl_decode_tuple!(A B C D E F G H I J K L M N O P Q R S);
impl_decode_tuple!(A B C D E F G H I J K L M N O P Q R S T);
// ^ Note: We make sure to support as many as parity-scale-codec's impls do.

/// This takes anything that can decode a stream if items and return an iterator over them.
fn decode_items_using<'a, 'scale, 'info, D: DecodeItemIterator<'scale, 'info>, T>(
	decoder: &'a mut D,
) -> impl Iterator<Item = Result<T, Error>> + 'a
where
	T: IntoVisitor,
	Error: From<<T::Visitor as Visitor>::Error>,
	D: DecodeItemIterator<'scale, 'info>,
{
	let mut idx = 0;
	core::iter::from_fn(move || {
		let item = decoder
			.decode_item(T::into_visitor())
			.map(|res| res.map_err(|e| Error::from(e).at_idx(idx)));
		idx += 1;
		item
	})
}

#[cfg(all(feature = "derive", feature = "primitive-types"))]
#[cfg(test)]
mod test {
	use super::*;
	use crate::{DecodeAsType, Field};
	use codec::Encode;

	/// Given a type definition, return type ID and registry representing it.
	fn make_type<T: scale_info::TypeInfo + 'static>() -> (u32, scale_info::PortableRegistry) {
		let m = scale_info::MetaType::new::<T>();
		let mut types = scale_info::Registry::new();
		let id = types.register_type(&m);
		let portable_registry: scale_info::PortableRegistry = types.into();

		(id.id, portable_registry)
	}

	// For most of our tests, we'll assert that whatever type we encode, we can decode back again to the given type.
	fn assert_encode_decode_to_with<T, A, B>(a: &A, b: &B)
	where
		A: Encode,
		B: DecodeAsType + PartialEq + std::fmt::Debug,
		T: scale_info::TypeInfo + 'static,
	{
		let (type_id, types) = make_type::<T>();
		let encoded = a.encode();
		let decoded =
			B::decode_as_type(&mut &*encoded, type_id, &types).expect("should be able to decode");
		assert_eq!(&decoded, b);
	}

	// Normally, the type info we want to use comes along with the type we're encoding.
	fn assert_encode_decode_to<A, B>(a: &A, b: &B)
	where
		A: Encode + scale_info::TypeInfo + 'static,
		B: DecodeAsType + PartialEq + std::fmt::Debug,
	{
		assert_encode_decode_to_with::<A, A, B>(a, b);
	}

	// Most of the time we'll just make sure that we can encode and decode back to the same type.
	fn assert_encode_decode_with<T, A>(a: &A)
	where
		A: Encode + DecodeAsType + PartialEq + std::fmt::Debug,
		T: scale_info::TypeInfo + 'static,
	{
		assert_encode_decode_to_with::<T, A, A>(a, a)
	}

	// Most of the time we'll just make sure that we can encode and decode back to the same type.
	fn assert_encode_decode<A>(a: &A)
	where
		A: Encode + scale_info::TypeInfo + 'static + DecodeAsType + PartialEq + std::fmt::Debug,
	{
		assert_encode_decode_to(a, a)
	}

	// Test that something can be encoded and then DecodeAsFields will work to decode it again.
	fn assert_encode_decode_as_fields<Foo>(foo: Foo)
	where
		Foo: scale_info::TypeInfo
			+ DecodeAsFields
			+ PartialEq
			+ std::fmt::Debug
			+ codec::Encode
			+ 'static,
	{
		let foo_encoded = foo.encode();
		let foo_encoded_cursor = &mut &*foo_encoded;

		let (ty, types) = make_type::<Foo>();

		let new_foo = match &types.resolve(ty).unwrap().type_def {
			scale_info::TypeDef::Composite(c) => {
				let mut field_iter =
					c.fields.iter().map(|f| Field::new(f.ty.id, f.name.as_deref()));
				Foo::decode_as_fields(foo_encoded_cursor, &mut field_iter, &types).unwrap()
			},
			scale_info::TypeDef::Tuple(t) => {
				let mut field_iter = t.fields.iter().map(|f| Field::unnamed(f.id));
				Foo::decode_as_fields(foo_encoded_cursor, &mut field_iter, &types).unwrap()
			},
			_ => {
				panic!("Expected composite or tuple type def")
			},
		};

		assert_eq!(foo, new_foo);
		assert_eq!(
			foo_encoded_cursor.len(),
			0,
			"leftover len when total was {}",
			foo_encoded.len()
		);
	}

	#[test]
	fn decode_primitives() {
		assert_encode_decode(&true);
		assert_encode_decode(&false);
		assert_encode_decode(&"hello".to_string());
	}

	#[test]
	fn decode_pointer_types() {
		assert_encode_decode_to(&true, &Box::new(true));
		assert_encode_decode_to(&true, &Arc::new(true));
		assert_encode_decode_to(&true, &Rc::new(true));
		assert_encode_decode_to(&true, &Cow::Borrowed(&true));
	}

	#[test]
	fn decode_duration() {
		assert_encode_decode_with::<(u64, u32), _>(&Duration::from_millis(12345));
	}

	#[test]
	fn decode_ranges() {
		assert_encode_decode(&(1..10));
		assert_encode_decode(&(1..=10));
	}

	#[test]
	fn decode_basic_numbers() {
		fn decode_all_types(n: u128) {
			assert_encode_decode_to(&n, &(n as u8));
			assert_encode_decode_to(&n, &(n as u16));
			assert_encode_decode_to(&n, &(n as u32));
			assert_encode_decode_to(&n, &(n as u64));
			assert_encode_decode_to(&n, &n);

			assert_encode_decode_to(&n, &(n as i8));
			assert_encode_decode_to(&n, &(n as i16));
			assert_encode_decode_to(&n, &(n as i32));
			assert_encode_decode_to(&n, &(n as i64));
			assert_encode_decode_to(&n, &(n as i128));
		}

		decode_all_types(0);
		decode_all_types(1);
		decode_all_types(127);
	}

	#[test]
	fn decode_cows() {
		let a = "hello";
		assert_encode_decode_to(&a, &Cow::<'_, str>::Borrowed(a));
		// Decoding a Cow means being able to jump into the inner composite type
		// (Cow's are a one-field composite type in TypeInfo by the looks of it).
		assert_encode_decode(&Cow::<'_, str>::Borrowed(a));
	}

	#[test]
	fn decode_sequences() {
		assert_encode_decode_to(&vec![1u8, 2, 3], &[1u8, 2, 3]);
		assert_encode_decode_to(&vec![1u8, 2, 3], &vec![1u8, 2, 3]);
		assert_encode_decode_to(&vec![1u8, 2, 3], &LinkedList::from_iter([1u8, 2, 3]));
		assert_encode_decode_to(&vec![1u8, 2, 3], &VecDeque::from_iter([1u8, 2, 3]));
		assert_encode_decode_to(&vec![1u8, 2, 3, 2], &BTreeSet::from_iter([1u8, 2, 3, 2]));
		// assert_encode_decode_to(&vec![1u8,2,3], &BinaryHeap::from_iter([1u8,2,3])); // No partialEq for BinaryHeap
	}

	#[test]
	fn decode_types_via_tuples_or_composites() {
		// Some type we know will be a composite type because we made it..
		#[derive(DecodeAsType, codec::Encode, scale_info::TypeInfo)]
		#[decode_as_type(crate_path = "crate")]
		struct Foo<A> {
			val: A,
		}

		// Make our own enum just to check that it can be decoded through tuples etc too:
		#[derive(DecodeAsType, codec::Encode, scale_info::TypeInfo, Debug, PartialEq, Clone)]
		#[decode_as_type(crate_path = "crate")]
		enum Wibble {
			Bar(u64),
		}

		fn check<A>(a: A)
		where
			A: Encode
				+ scale_info::TypeInfo
				+ 'static
				+ DecodeAsType
				+ PartialEq
				+ std::fmt::Debug
				+ Clone,
		{
			let tup = ((a.clone(),),);
			let struc = Foo { val: Foo { val: a.clone() } };

			assert_encode_decode_to(&tup, &a);
			assert_encode_decode_to(&struc, &a);
		}

		// All of these types can be decoded through nested
		// tuples or composite types that have exactly one field.
		check(123u8);
		check(123u16);
		check(123u32);
		check(123u64);
		check(123u128);
		check(123i8);
		check(123i16);
		check(123i32);
		check(123i64);
		check(123i128);
		check(true);
		check("hello".to_string());
		check(Bits::from_iter([true, false, true, true]));
		check([1, 2, 3, 4, 5]);
		check(vec![1, 2, 3, 4, 5]);
		check(NonZeroU8::new(100).unwrap());
		check(Some(123));
		check(Ok::<_, bool>(123));
		check(Wibble::Bar(12345));
	}

	#[test]
	fn decode_tuples() {
		// Some struct with the same shape as our tuples.
		#[derive(DecodeAsType, codec::Encode, scale_info::TypeInfo, Debug, PartialEq, Clone)]
		#[decode_as_type(crate_path = "crate")]
		struct Foo {
			a: u8,
			b: u16,
			c: bool,
		}

		// Decode to the same:
		assert_encode_decode(&(1u8, 2u16, true));
		// Decode to struct of similar shape:
		assert_encode_decode_to(&(1u8, 2u8, true), &Foo { a: 1u8, b: 2u16, c: true });
		// Decode from struct of similar shape:
		assert_encode_decode_to(&Foo { a: 1u8, b: 2u16, c: true }, &(1u8, 2u8, true));
	}

	#[test]
	fn decode_composites_to_tuples() {
		#[derive(codec::Encode, scale_info::TypeInfo)]
		struct Foo {
			hello: bool,
			other: (u8, u32),
		}

		let input = Foo { hello: true, other: (1, 3) };
		// Same:
		assert_encode_decode_to(&input, &(true, (1u8, 3u32)));
		// Different:
		assert_encode_decode_to(&input, &(true, (1u64, 3u64)));
	}

	#[test]
	fn decode_compacts() {
		assert_encode_decode(&Compact(126u64));
	}

	#[test]
	fn decode_options_and_results() {
		// These are hardcoded so let's make sure they work..
		assert_encode_decode(&Some(123i128));
		assert_encode_decode(&(None as Option<bool>));
		assert_encode_decode(&Ok::<_, bool>(123i128));
		assert_encode_decode(&Err::<bool, _>(123i128));
	}

	#[test]
	fn decode_bits() {
		assert_encode_decode(&Bits::new());
		assert_encode_decode(&Bits::from_iter([true, false, false, true, false]));
	}

	#[test]
	#[cfg(feature = "primitive-types")]
	fn decode_hxxx() {
		use ::primitive_types::{H128, H160, H256, H384, H512, H768};

		fn try_decode_hxxx(input: impl IntoIterator<Item = u8>) {
			let mut bytes: Vec<u8> = input.into_iter().collect();

			macro_rules! check_ty {
				($bytes:expr, $bits:literal, $ty:ident) => {
					while $bytes.len() < $bits / 8 {
						$bytes.push(0)
					}
					assert_encode_decode(&$ty::from_slice(&$bytes));
					assert_encode_decode_to(&$ty::from_slice(&$bytes), &$bytes);
					assert_encode_decode_to(&$bytes, &$ty::from_slice(&$bytes));
				};
			}
			check_ty!(bytes, 128, H128);
			check_ty!(bytes, 160, H160);
			check_ty!(bytes, 256, H256);
			check_ty!(bytes, 384, H384);
			check_ty!(bytes, 512, H512);
			check_ty!(bytes, 768, H768);
		}

		try_decode_hxxx([0]);
		try_decode_hxxx([1, 2, 3, 4]);
		try_decode_hxxx([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
	}

	#[test]
	fn decoding_can_skip_named_struct_fields() {
		#[derive(DecodeAsType, PartialEq, Debug)]
		#[decode_as_type(crate_path = "crate")]
		struct Foo {
			some_field: u8,
			value: u16,
			#[decode_as_type(skip)]
			some_field_to_skip: bool,
			#[decode_as_type(skip)]
			other_field_to_skip: usize,
		}

		#[derive(codec::Encode, scale_info::TypeInfo)]
		struct FooPartial {
			some_field: u8,
			value: u16,
		}

		assert_encode_decode_to(
			&FooPartial { some_field: 123, value: 456 },
			&Foo {
				some_field: 123,
				value: 456,
				// fields will be defaulted if skipped:
				some_field_to_skip: false,
				other_field_to_skip: 0,
			},
		);
	}

	#[test]
	fn decoding_can_skip_unnamed_struct_fields() {
		#[derive(DecodeAsType, PartialEq, Debug)]
		#[decode_as_type(crate_path = "crate")]
		struct Foo(u8, #[decode_as_type(skip)] bool, #[decode_as_type(skip)] usize);

		#[derive(codec::Encode, scale_info::TypeInfo)]
		struct FooPartial {
			some_field: u8,
		}

		assert_encode_decode_to(
			&FooPartial { some_field: 123 },
			&Foo(
				123, // fields will be defaulted if skipped:
				false, 0,
			),
		);
	}

	#[test]
	fn decoding_can_skip_enum_variant_fields() {
		#[derive(DecodeAsType, PartialEq, Debug)]
		#[decode_as_type(crate_path = "crate")]
		enum Foo {
			NamedField {
				some_field: u8,
				#[decode_as_type(skip)]
				some_field_to_skip: bool,
				// the codec attr should work too:
				#[codec(skip)]
				another_field_to_skip: String,
				value: u16,
			},
			UnnamedField(bool, #[decode_as_type(skip)] usize, String),
		}

		#[derive(codec::Encode, scale_info::TypeInfo)]
		enum FooPartial {
			NamedField { some_field: u8, value: u16 },
			UnnamedField(bool, String),
		}

		assert_encode_decode_to(
			&FooPartial::NamedField { some_field: 123, value: 456 },
			&Foo::NamedField {
				some_field: 123,
				some_field_to_skip: false,
				another_field_to_skip: String::new(),
				value: 456,
			},
		);
		assert_encode_decode_to(
			&FooPartial::UnnamedField(true, "hello".to_string()),
			&Foo::UnnamedField(true, 0, "hello".to_string()),
		);
	}

	#[test]
	fn decode_as_fields_works() {
		use std::fmt::Debug;

		#[derive(DecodeAsType, codec::Encode, PartialEq, Debug, scale_info::TypeInfo)]
		#[decode_as_type(crate_path = "crate")]
		struct Foo {
			some_field: u8,
			value: u16,
		}

		assert_encode_decode_as_fields(Foo { some_field: 123, value: 456 });

		#[derive(DecodeAsType, codec::Encode, PartialEq, Debug, scale_info::TypeInfo)]
		#[decode_as_type(crate_path = "crate")]
		struct Foo2(String, bool, u64);

		assert_encode_decode_as_fields(Foo2("hello".to_string(), true, 12345));

		#[derive(DecodeAsType, codec::Encode, PartialEq, Debug, scale_info::TypeInfo)]
		#[decode_as_type(crate_path = "crate")]
		struct Foo3;

		assert_encode_decode_as_fields(Foo3);

		// Tuples should work, too:
		assert_encode_decode_as_fields((true, 123u8, "hello".to_string()));
	}
}
