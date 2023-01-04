use crate::semantics::operations::{
    filter::Filter, join::Join, leftjoin::LeftJoin, limit::Limit, minus::Minus, offset::Offset,
    projection::Projection, scan::Scan, union::Union, Operation, OperationVisitor,
};

pub(crate) enum FlattenError {
    NonConjunctiveStructure,
}

pub(crate) struct Flatten;
impl Flatten {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

type FlattenResult<'a, S, J, M, L> = Result<Vec<Operation<'a, S, J, M, L>>, FlattenError>;

impl<'a, S: Clone, J, M, L> OperationVisitor<'a, S, J, M, L, FlattenResult<'a, S, J, M, L>>
    for Flatten
{
    fn visit_scan(
        &mut self,
        o: &'a Scan<S, J, M, L>,
    ) -> Result<Vec<Operation<'a, S, J, M, L>>, FlattenError> {
        Ok(vec![Operation::Scan(o.to_owned())])
    }

    fn visit_join(
        &mut self,
        o: &'a Join<J, Operation<'a, S, J, M, L>>,
    ) -> FlattenResult<'a, S, J, M, L> {
        let mut left = self.visit(&*o.left)?;
        let mut right = self.visit(&*o.right)?;

        left.append(&mut right);

        Ok(left)
    }

    fn visit_projection(
        &mut self,
        _o: &'a Projection<Operation<'a, S, J, M, L>>,
    ) -> FlattenResult<'a, S, J, M, L> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_union(
        &mut self,
        _o: &'a Union<Operation<'a, S, J, M, L>>,
    ) -> FlattenResult<'a, S, J, M, L> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_filter(
        &mut self,
        _o: &'a Filter<Operation<'a, S, J, M, L>>,
    ) -> FlattenResult<'a, S, J, M, L> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_leftjoin(
        &mut self,
        _o: &'a LeftJoin<Operation<'a, S, J, M, L>>,
    ) -> FlattenResult<'a, S, J, M, L> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_minus(
        &mut self,
        _o: &'a Minus<M, Operation<'a, S, J, M, L>>,
    ) -> FlattenResult<'a, S, J, M, L> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_offset(
        &mut self,
        _o: &'a Offset<Operation<'a, S, J, M, L>>,
    ) -> FlattenResult<'a, S, J, M, L> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_limit(
        &mut self,
        _o: &'a Limit<L, Operation<'a, S, J, M, L>>,
    ) -> FlattenResult<'a, S, J, M, L> {
        Err(FlattenError::NonConjunctiveStructure)
    }
}
