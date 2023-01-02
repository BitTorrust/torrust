mod pieces_selection;
pub mod simple_selection;

pub use pieces_selection::PiecesSelection;
pub use simple_selection::SimpleSelector;

pub mod distributed_selection;
pub use distributed_selection::DistributedSelector;
