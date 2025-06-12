use crate::segment::Segment;

pub enum DirectiveEvaluation<'a> {
    Segments { segments: Vec<&'a Segment> },
}
