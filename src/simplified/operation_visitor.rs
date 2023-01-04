use super::operation::{Join, Operation, Scan};

trait OperationVisitor<'a, T> {
    fn visit(&mut self, o: &'a Operation<'a>) -> T {
        match o {
            Operation::Scan(s) => self.visit_scan(s),
            Operation::Join(j) => self.visit_join(j),
            // More operations omitted
        }
    }

    fn visit_scan(&mut self, o: &'a Scan<'a>) -> T;
    fn visit_join(&mut self, o: &'a Join<'a>) -> T;
}
