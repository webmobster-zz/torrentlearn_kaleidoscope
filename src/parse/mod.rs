pub use self::parsetree_gen::ParseTree;
pub use self::parsetree_gen::Data;
pub use self::parsetree_gen::Statement;
pub use self::parsetree_gen::Position;


mod parsetree_gen;
mod parsetree_ungen;

#[derive(Copy,Clone)]
pub enum SingleOperators
{
    Add
}
#[derive(Copy,Clone)]
pub enum VecOperators
{
    Add
}
#[derive(Copy,Clone)]
pub enum MapOperators
{
    Add
}
#[derive(Copy,Clone)]
pub enum ReduceOperators
{
    Add
}
