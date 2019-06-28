#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

//#[cfg(not(feature = "idebuild"))]
//mod linputbindings {
//    include!(concat!(env!("OUT_DIR"), "/linux-input.rs"));
//}

//#[cfg(feature = "idebuild")]
mod linputbindings;

pub use self::linputbindings::*;
