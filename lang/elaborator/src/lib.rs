pub mod conversion_checking;
pub mod index_unification;
pub mod normalizer;
pub mod result;
pub mod typechecker;

pub use typechecker::type_info_table::build::build_type_info_table;
pub use typechecker::type_info_table::ModuleTypeInfoTable;
pub use typechecker::type_info_table::TypeInfoTable;
