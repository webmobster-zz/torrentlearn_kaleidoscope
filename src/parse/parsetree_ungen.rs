use super::SingleOperators;
use super::VecOperators;
use super::MapOperators;
use super::ReduceOperators;
pub enum StatementUngen
{
    //dest, source
    SingleStatement(SingleOperators,PositionUngen,DataUngen),
    //dest low, source low, length both
    VecStatement(VecOperators,PositionUngen,PositionUngen,PositionUngen),
    //dest low, length dest, source
    MapStatement(SingleOperators,PositionUngen,PositionUngen,DataUngen),
    //dest, source low, source length
    ReduceStatement(ReduceOperators,PositionUngen,DataUngen,DataUngen)
}


pub enum DataUngen
{
    Val,
    Pos(PositionUngen)
}
pub enum PositionUngen
{
    ConstPos,
    //Is this the first, second, etc argument
    VarPos(u8),
}
