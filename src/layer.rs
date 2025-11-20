// Layer scaffolding for grammar of graphics

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::geom::Geom;

/// Statistical transformation
#[derive(Clone)]
pub enum Stat {
	Identity,
	Count,
	Bin,
	Smooth,
	// Add more as needed
}

/// Position adjustment for overlapping geoms
#[derive(Clone)]
pub enum Position {
	Identity,
	Stack,
	Dodge,
	Jitter,
	Fill,
	// Add more as needed
}

/// Layer struct - represents one layer in a plot
/// Each layer has its own geom, optional data, aesthetic mappings, stat, and position
pub struct Layer {
	pub geom: Box<dyn Geom>,
	pub data: Option<Box<dyn DataSource>>,
	pub mapping: AesMap,
	pub stat: Stat,
	pub position: Position,
}
