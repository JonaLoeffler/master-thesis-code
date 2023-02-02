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

type FlattenResult<'a> = Result<Vec<Operation<'a>>, FlattenError>;

impl<'a> OperationVisitor<'a, FlattenResult<'a>> for Flatten {
    fn visit_scan(&mut self, o: &'a Scan<'a>) -> FlattenResult<'a> {
        Ok(vec![Operation::Scan(o.to_owned())])
    }

    fn visit_join(&mut self, o: &'a Join<Operation<'a>>) -> FlattenResult<'a> {
        let mut left = self.visit(&o.left)?;
        let mut right = self.visit(&o.right)?;

        left.append(&mut right);

        Ok(left)
    }

    fn visit_projection(&mut self, _o: &'a Projection<Operation<'a>>) -> FlattenResult<'a> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_union(&mut self, _o: &'a Union<Operation<'a>>) -> FlattenResult<'a> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_filter(&mut self, _o: &'a Filter<Operation<'a>>) -> FlattenResult<'a> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_leftjoin(&mut self, _o: &'a LeftJoin<Operation<'a>>) -> FlattenResult<'a> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_minus(&mut self, _o: &'a Minus<Operation<'a>>) -> FlattenResult<'a> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_offset(&mut self, _o: &'a Offset<Operation<'a>>) -> FlattenResult<'a> {
        Err(FlattenError::NonConjunctiveStructure)
    }

    fn visit_limit(&mut self, _o: &'a Limit<Operation<'a>>) -> FlattenResult<'a> {
        Err(FlattenError::NonConjunctiveStructure)
    }
}
