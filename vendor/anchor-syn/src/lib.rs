//! Patched minimal anchor-syn 0.30.1 replacement to avoid using Span::source_file.
//! Only provides the symbols anchor-lang proc-macros link against for IDL build.
pub mod idl;

// Re-export minimal items expected by anchor-lang's use of anchor-syn.
pub mod parser {
	pub mod docs { pub fn parse<T>(_t:&[T])->Option<Vec<String>>{ None } }
	pub mod context { pub struct CrateContext; impl CrateContext { pub fn parse(_p:&std::path::Path)->Result<Self,()> { Err(()) } pub fn type_aliases(&self)->std::vec::IntoIter<syn::ItemType>{ vec![].into_iter() } pub fn structs(&self)->std::vec::IntoIter<syn::ItemStruct>{ vec![].into_iter() } pub fn enums(&self)->std::vec::IntoIter<syn::ItemEnum>{ vec![].into_iter() } }
}
pub mod idl_build {
	// Anchor expects a trait IdlBuild with certain fns; provide no-op.
	pub trait IdlBuild {
		fn create_type() -> Option<String> { None }
		fn insert_types(_types: &mut std::collections::BTreeMap<String, String>) {}
		fn get_full_path() -> String { String::new() }
	}
}
