/*========================================================================*\
** NotVeryMoe BevyUtil | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*========================================================================*/

#[cfg(test)] mod test;

pub use shrinkwraprs::Shrinkwrap;

pub trait CompactStr: Default + Copy + Eq + Ord + std::hash::Hash + std::fmt::Debug + std::fmt::Display {
    type InnerType: num_traits::PrimInt;

    const CAPACITY: usize;

    fn new(value: &str) -> Self;
    fn try_new(value: &str) -> Result<Self, &'static str>;
    fn to_str(self) -> String;
    fn len(self) -> usize;
    fn is_empty(self) -> bool;
    fn try_from_raw(value: Self::InnerType) -> Option<Self>;
    fn from_raw(value: Self::InnerType) -> Self;

    /// Creates a new CompactStr from its raw representation without
    /// validating length or 
    /// 
    /// # Safety
    /// There's no risk of UB in calling this. However, if the value
    /// contains invalid codepoints or a null in the middle of the
    /// raw version, then, behaviour of the struct is unspecified.
    unsafe fn from_raw_unchecked(value: Self::InnerType) -> Self;

    fn to_raw(self) -> Self::InnerType;
    fn set_ascii(&mut self, idx: usize, value: u8);
    fn get_ascii(&self, idx: usize) -> u8;
}

macro_rules! set_raw {
    ($T:ty, $s:ident, $idx:ident, $val:ident) => {{
        let msk: $T = 1 << ($idx*5);
        let val: $T = ($val & 0b1_1111) as $T << ($idx*5);
        ($s & !msk) | val
    }}
}

macro_rules! impl_compact_str {
    ($S:ident, $T:ty) => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $S($T);

        impl std::fmt::Debug for $S {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("CompactStr").field(&self.to_str()).finish()
            }
        }
        
        impl std::fmt::Display for $S {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.to_str())
            }
        }

        impl $S {
            pub const CAPACITY: usize = std::mem::size_of::<$T>()*8/5;

            #[inline] pub const fn new(value: &str) -> Self {
                match Self::try_new(value) {
                    Ok(value) => value,
                    Err(e)    => panic!("{}", e) ,
                }
            }

            #[inline] pub const fn try_new(value: &str) -> Result<Self, &'static str> {
                let chars = value.as_bytes();
                if chars.len() > Self::CAPACITY { 
                    return Err("String contains too many characters."); 
                }

                let mut result: $T = 0;
                let mut i = 0;
                while i < chars.len() {
                    if let Some(val) = encode_char(chars[i]) {
                        result = set_raw!($T, result, i, val);
                        i += 1;
                    } else {
                        return Err("String contains invalid characters.");
                    }
                }

                Ok(Self(result))
            }

            #[inline] pub fn to_str(self) -> String {
                let mut result = Vec::with_capacity(self.len());
                for i in 0..self.len() {
                    result.push(self.get_ascii(i));
                }
                unsafe{ String::from_utf8_unchecked(result) }
            }

            #[inline] pub const fn len(self) -> usize {
                let mut i = 0;
                while i < Self::CAPACITY {
                    if self.get_raw(i) == 0 { return i; }
                    i += 1;
                }
                Self::CAPACITY
            }

            #[inline] pub const fn is_empty(self) -> bool {
                self.get_raw(0) == 0
            }

            #[inline] pub const fn try_from_raw(value: $T) -> Option<Self> {
                let result = Self(value);
                let len = result.len();
                let mut i = 0;
                while i < Self::CAPACITY {
                    let raw = result.get_raw(i);
                    if raw > {if i < len { 27 } else { 0 }} { return None; }
                    i += 1;
                }
                Some(result)
            }

            #[inline] pub const fn from_raw(value: $T) -> Self {
                if let Some(value) = Self::try_from_raw(value) {
                    value
                } else {
                    panic!("Invalid raw representation of CompactStr");
                }
            }

            /// Creates a new CompactStr from its raw representation without
            /// validating length or 
            /// 
            /// # Safety
            /// There's no risk of UB in calling this. However, if the value
            /// contains invalid codepoints or a null in the middle of the
            /// raw version, then, behaviour of the struct is unspecified.
            #[inline] pub const unsafe fn from_raw_unchecked(value: $T) -> Self {
                Self(value)
            }

            #[inline] pub const fn to_raw(self) -> $T {
                self.0
            }

            #[inline] pub fn set_ascii(&mut self, idx: usize, value: u8) {
                assert!(idx < self.len(), "Attempt to index out of range");
                self.set_raw(idx, encode_char(value).expect("Invalid character"));
            }

            #[inline] pub const fn get_ascii(&self, idx: usize) -> u8 {
                assert!(idx < self.len(), "Attempt to index out of range");
                unencode_char(self.get_raw(idx))
            }

            #[inline] fn set_raw(&mut self, idx: usize, val: u8) {
                let s = self.0;
                self.0 = set_raw!($T, s, idx, val)
            }

            #[inline] const fn get_raw(self, idx: usize) -> u8 {
                (self.0 >> (idx*5) & 0b1_1111) as u8
            }
        }

        impl CompactStr for $S {
            type InnerType = $T;

            const CAPACITY: usize = Self::CAPACITY;
        
            fn new(value: &str) -> Self { Self::new(value) }
            fn try_new(value: &str) -> Result<Self, &'static str> { Self::try_new(value) }
            fn to_str(self) -> String { Self::to_str(self) }
            fn len(self) -> usize { Self::len(self) }
            fn is_empty(self) -> bool { Self::is_empty(self) }
            fn try_from_raw(value: Self::InnerType) -> Option<Self> { Self::try_from_raw(value) }
            fn from_raw(value: Self::InnerType) -> Self { Self::from_raw(value) }
            unsafe fn from_raw_unchecked(value: Self::InnerType) -> Self { Self::from_raw_unchecked(value) }
            fn to_raw(self) -> Self::InnerType { Self::to_raw(self) }
            fn set_ascii(&mut self, idx: usize, value: u8) { Self::set_ascii(self, idx, value) }
            fn get_ascii(&self, idx: usize) -> u8 { Self::get_ascii(self, idx) }
        }
    };
}

#[macro_export]
macro_rules! newtype_compactstr {
    ($V:vis, $S:ident, $T:ty) => {
        #[derive(compact_str::Shrinkwrap, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash, std::fmt::Debug)]
        #[repr(transparent)]
        $V struct $S($T);

        impl $S {
            $V const fn new(v: &str) -> Self {
                Self(<$T>::new(v))
            }

            $V const fn try_new(v: &str) -> Result<Self, &'static str> {
                match <$T>::try_new(v) {
                    Ok(v)  => Ok(Self(v)),
                    Err(v) => Err(v),
                }
            }

            $V const fn try_from_raw(value: <$T as compact_str::CompactStr>::InnerType) -> Option<Self> {
                if let Some(v) = <$T>::try_from_raw(value) {
                    Some(Self(v))
                } else {
                    None
                }
            }

            $V const fn from_raw(value: <$T as compact_str::CompactStr>::InnerType) -> Self {
                Self(<$T>::from_raw(value))
            }
            
            $V const unsafe fn from_raw_unchecked(v: <$T as compact_str::CompactStr>::InnerType) -> Self {
                Self(<$T>::from_raw_unchecked(v))
            }
        }
        
        impl std::fmt::Display for $S {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.to_str())
            }
        }
    };
}

impl_compact_str!(CompactStr8,   u8  );
impl_compact_str!(CompactStr16,  u16 );
impl_compact_str!(CompactStr32,  u32 );
impl_compact_str!(CompactStr64,  u64 );
impl_compact_str!(CompactStr128, u128);

const fn encode_char(value: u8) -> Option<u8> {
    match value {
        b'a'..=b'z' => Some((value as u8)-95), /* A-Z -> 2-27 */ 
        b'A'..=b'Z' => Some((value as u8)-63), /* a-z -> 2-27*/ 
        b' '|b'_'   => Some(1               ), /* Spc */
        _ => None
    }
}

const fn unencode_char(value: u8) -> u8 {
    match value {
        0 => b'\0',
        1 => b'_',
        _ => (value + 63)
    }
}