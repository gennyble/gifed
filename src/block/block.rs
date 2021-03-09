use crate::block::Version;

use super::ScreenDescriptor;


pub enum Block {
	Version(Version),
	LogicalScreenDescriptor(ScreenDescriptor)
}