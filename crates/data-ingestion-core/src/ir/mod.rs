pub mod model;
pub mod normalizer;

pub use model::{
    IrArray, IrConstraint, IrDocument, IrEnum, IrField, IrNode, IrObject, IrType, SourceFormat,
};
pub use normalizer::IrNormalizer;
