use crate::generic_graph::{GenericGraph, GenericResult, GenericSchema};

pub type TestSchema = GenericSchema<usize, usize>;
pub type TestGraph = GenericGraph<usize, usize, usize, usize>;
pub type TestResult<T> = GenericResult<T, usize, usize, usize, usize>;