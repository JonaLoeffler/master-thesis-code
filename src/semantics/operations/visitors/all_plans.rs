use itertools::Itertools;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use std::{error::Error, fmt::Display};

use crate::semantics::operations::{
    filter::Filter,
    join::{CollJoin, IterJoin, Join, NewJoin},
    leftjoin::LeftJoin,
    limit::{CollLimit, IterLimit, Limit, NewLimit},
    minus::{CollMinus, IterMinus, Minus, NewMinus},
    offset::Offset,
    projection::Projection,
    scan::{CollScan, IterScan, Scan},
    union::Union,
    Operation, OperationVisitor,
};

use super::flatten::Flatten;

pub(crate) struct AllPlans<'a, S, J, M, L> {
    join: NewJoin<'a, S, J, M, L>,
    minus: NewMinus<'a, S, J, M, L>,
    limit: NewLimit<'a, S, J, M, L>,
}

impl<'a> AllPlans<'a, CollScan, CollJoin, CollMinus, CollLimit> {
    pub(crate) fn coll() -> Self {
        Self {
            join: Join::collection,
            minus: Minus::collection,
            limit: Limit::collection,
        }
    }
}

impl<'a> AllPlans<'a, IterScan<'a>, IterJoin, IterMinus, IterLimit> {
    pub(crate) fn iter() -> Self {
        Self {
            join: Join::iterator,
            minus: Minus::iterator,
            limit: Limit::iterator,
        }
    }
}

type AllPlansResult<'a, S, J, M, L> = Result<Vec<Operation<'a, S, J, M, L>>, AllPlansError>;
#[derive(Debug)]
pub enum AllPlansError {
    UnexpectedOperation,
}
impl Display for AllPlansError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllPlansError::UnexpectedOperation => {
                f.write_str(&format!("Unexpected operation in plan"))
            }
        }
    }
}
impl Error for AllPlansError {}

impl<'a, S, J, M, L> OperationVisitor<'a, S, J, M, L, AllPlansResult<'a, S, J, M, L>>
    for AllPlans<'a, S, J, M, L>
where
    S: Clone + PartialEq,
    J: Clone + PartialEq,
    M: Clone + PartialEq,
    L: Clone + PartialEq,
{
    fn visit(&mut self, o: &'a Operation<'a, S, J, M, L>) -> AllPlansResult<'a, S, J, M, L> {
        if let Ok(ops) = Flatten::new().visit(o) {
            let scans: Vec<Scan<'a, S, J, M, L>> = ops
                .iter()
                .map(|o| match o {
                    Operation::Scan(s) => Ok(s.clone()),
                    _ => Err(AllPlansError::UnexpectedOperation),
                })
                .collect::<Result<Vec<Scan<'a, S, J, M, L>>, AllPlansError>>()?;

            let trees: Vec<Rc<RefCell<TreeNode>>> = TreeNode::all_possible_fbt(2 * scans.len() - 1)
                .iter()
                .filter_map(|t| {
                    t.as_ref()
                        .unwrap()
                        .as_ref()
                        .borrow()
                        .enumerate_leafs(&mut 0)
                })
                .collect();

            log::info!(
                "Number of trees: {}, for {} scans",
                trees.len(),
                scans.len()
            );

            let scan_permutations: Vec<HashMap<usize, Scan<'a, S, J, M, L>>> = scans
                .iter()
                .permutations(scans.len())
                .map(|scans| {
                    let mut res: HashMap<usize, Scan<S, J, M, L>> = HashMap::new();

                    for (i, s) in scans.iter().enumerate() {
                        res.insert(i + 1, s.clone().clone());
                    }

                    res
                })
                .collect();

            log::info!("Number of scan permutations: {}", scan_permutations.len());

            let ops: Vec<Operation<S, J, M, L>> = trees
                .into_iter()
                .cartesian_product(scan_permutations.iter())
                .map(|(t, s)| t.as_ref().borrow().to_operation(s, self.join))
                .collect();

            log::info!("Number of execution plans: {}", ops.len());

            return Ok(ops);
        }

        match o {
            Operation::Scan(s) => self.visit_scan(s),
            Operation::Join(j) => self.visit_join(j),
            Operation::Projection(p) => self.visit_projection(p),
            Operation::Union(u) => self.visit_union(u),
            Operation::Filter(f) => self.visit_filter(f),
            Operation::LeftJoin(l) => self.visit_leftjoin(l),
            Operation::Minus(m) => self.visit_minus(m),
            Operation::Offset(o) => self.visit_offset(o),
            Operation::Limit(l) => self.visit_limit(l),
        }
    }

    fn visit_scan(&mut self, _: &'a Scan<'a, S, J, M, L>) -> AllPlansResult<'a, S, J, M, L> {
        panic!("Should have optimized before now")
    }

    fn visit_join(
        &mut self,
        _: &'a Join<J, Operation<'a, S, J, M, L>>,
    ) -> AllPlansResult<'a, S, J, M, L> {
        panic!("Should have optimized before now")
    }

    fn visit_projection(
        &mut self,
        o: &'a Projection<Operation<'a, S, J, M, L>>,
    ) -> AllPlansResult<'a, S, J, M, L> {
        Ok(self
            .visit(&o.operation)?
            .into_iter()
            .map(|op| Operation::Projection(Projection::new(op, o.vars.to_owned())))
            .collect())
    }

    fn visit_union(
        &mut self,
        o: &'a Union<Operation<'a, S, J, M, L>>,
    ) -> AllPlansResult<'a, S, J, M, L> {
        Ok(self
            .visit(&o.left)?
            .into_iter()
            .cartesian_product(self.visit(&o.right)?.into_iter())
            .map(|(l, r)| Operation::Union(Union::new(l, r)))
            .collect())
    }

    fn visit_filter(
        &mut self,
        o: &'a Filter<Operation<'a, S, J, M, L>>,
    ) -> AllPlansResult<'a, S, J, M, L> {
        Ok(self
            .visit(&o.operation)?
            .into_iter()
            .map(|op| Operation::Filter(Filter::new(op, *o.condition.to_owned())))
            .collect())
    }

    fn visit_leftjoin(
        &mut self,
        o: &'a LeftJoin<Operation<'a, S, J, M, L>>,
    ) -> AllPlansResult<'a, S, J, M, L> {
        Ok(self
            .visit(&o.left)?
            .into_iter()
            .cartesian_product(self.visit(&o.right)?.into_iter())
            .map(|(l, r)| Operation::LeftJoin(LeftJoin::new(l, r, self.join, self.minus)))
            .collect())
    }

    fn visit_minus(
        &mut self,
        o: &'a Minus<M, Operation<'a, S, J, M, L>>,
    ) -> AllPlansResult<'a, S, J, M, L> {
        Ok(self
            .visit(&o.left)?
            .into_iter()
            .cartesian_product(self.visit(&o.right)?.into_iter())
            .map(|(l, r)| Operation::Minus((self.minus)(l, r)))
            .collect())
    }

    fn visit_offset(
        &mut self,
        o: &'a Offset<Operation<'a, S, J, M, L>>,
    ) -> AllPlansResult<'a, S, J, M, L> {
        Ok(self
            .visit(&o.operation)?
            .into_iter()
            .map(|op| Operation::Offset(Offset::new(op, o.offset)))
            .collect())
    }

    fn visit_limit(
        &mut self,
        o: &'a crate::semantics::operations::limit::Limit<L, Operation<'a, S, J, M, L>>,
    ) -> AllPlansResult<'a, S, J, M, L> {
        Ok(self
            .visit(&o.operation)?
            .into_iter()
            .map(|op| Operation::Limit((self.limit)(op, o.limit)))
            .collect())
    }
}

#[derive(Debug, PartialEq, Clone)]
struct TreeNode {
    val: usize,
    left: Option<Rc<RefCell<TreeNode>>>,
    right: Option<Rc<RefCell<TreeNode>>>,
}

type TreeNodeResult = Vec<Option<Rc<RefCell<TreeNode>>>>;

impl TreeNode {
    fn new(val: usize) -> Self {
        Self {
            val,
            left: None,
            right: None,
        }
    }

    fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    fn enumerate_leafs(&self, n: &mut usize) -> Option<Rc<RefCell<TreeNode>>> {
        let node = if self.is_leaf() {
            *n += 1;
            TreeNode::new(n.clone())
        } else {
            let mut node = TreeNode::new(0);
            node.left = self.left.as_ref().unwrap().borrow().enumerate_leafs(n);
            node.right = self.right.as_ref().unwrap().borrow().enumerate_leafs(n);
            node
        };

        Some(Rc::new(RefCell::new(node)))
    }

    fn to_operation<'a, S: Clone, J, M, L>(
        &self,
        scans: &HashMap<usize, Scan<'a, S, J, M, L>>,
        join: NewJoin<'a, S, J, M, L>,
    ) -> Operation<'a, S, J, M, L> {
        if self.is_leaf() {
            Operation::Scan(scans.get(&self.val).unwrap().clone())
        } else {
            let left = self
                .left
                .as_ref()
                .unwrap()
                .borrow()
                .to_operation(scans, join);

            let right = self
                .right
                .as_ref()
                .unwrap()
                .borrow()
                .to_operation(scans, join);

            Operation::Join((join)(left, right))
        }
    }

    fn all_possible_fbt(n: usize) -> TreeNodeResult {
        type Cache = HashMap<usize, TreeNodeResult>;

        log::info!(
            "Attempting to find all possible binary trees with {n} nodes ({} leafs)",
            (n + 1) / 2
        );

        fn helper(i: usize, cache: &mut Cache) -> &TreeNodeResult {
            if !cache.contains_key(&i) {
                let mut ans = vec![];
                if i == 1 {
                    ans.push(Some(Rc::new(RefCell::new(TreeNode::new(0)))));
                } else {
                    for k in (1..i - 1).step_by(2) {
                        let left = helper(k, cache).clone();
                        let right = helper(i - 1 - k, cache);

                        for nl in left {
                            for nr in right {
                                ans.push(Some(Rc::new(RefCell::new(TreeNode {
                                    val: 0,
                                    left: nl.clone(),
                                    right: nr.clone(),
                                }))))
                            }
                        }
                    }
                }
                cache.insert(i, ans.clone());
            }
            return cache.get(&i).unwrap();
        }
        let mut cache: Cache = HashMap::new();
        helper(n, &mut cache).clone()
    }
}
