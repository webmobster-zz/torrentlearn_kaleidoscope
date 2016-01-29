use torrentlearn_model::UUID;
use super::SingleOperators;
use super::VecOperators;
use super::MapOperators;
use super::ReduceOperators;

pub enum ParseTree
{
    ListStatement(Vec<Statement>),
    SomeStatement(Statement),
}


pub enum Statement
{
    //dest, source
    SingleStatement(SingleOperators,Position,Data),
    //dest low, source low, length both
    VecStatement(VecOperators,Position,Position,Position),
    //dest low, length dest, source
    MapStatement(MapOperators,Position,Position,Data),
    //dest, source low, source length
    ReduceStatement(ReduceOperators,Position,Data,Data)
}


pub enum Data
{
    Val(u64),
    Pos(Position)
}
pub enum Position
{
    ConstPos(u64),
    //Is this the first, second, etc argument
    VarPos(u8),
}
