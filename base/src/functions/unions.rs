use crate::{expressions::parser::Node, functions::Function};

/// How many areas a reference has: `(A1,B2:C3)` has 2.
pub(crate) fn count_areas(node: &Node) -> usize {
    match node {
        Node::OpUnionKind(areas) => areas.iter().map(count_areas).sum(),
        _ => 1,
    }
}

fn flatten_union(node: &Node, out: &mut Vec<Node>) {
    match node {
        Node::OpUnionKind(areas) => {
            for area in areas {
                flatten_union(area, out);
            }
        }
        _ => out.push(node.clone()),
    }
}

/// Rewrites a call whose arguments hold a union of references (`(A1,C1:C3)`)
/// into the call Excel means by it: in a function that takes a list of values
/// each area becomes an argument of its own, `SUM((A1,C1))` is `SUM(A1,C1)`;
/// in INDEX the area picked by `area_num` becomes the reference.
/// Returns None where a union is not allowed, and the function then sees it
/// as #VALUE!.
pub(crate) fn expand_unions(kind: &Function, args: &[Node]) -> Option<Vec<Node>> {
    // Arguments before this position are not references.
    let first = match kind {
        Function::Sum
        | Function::Sumsq
        | Function::Product
        | Function::Count
        | Function::Counta
        | Function::Average
        | Function::Averagea
        | Function::Max
        | Function::MaxA
        | Function::Min
        | Function::MinA
        | Function::Stdev
        | Function::StDevP
        | Function::StDevS
        | Function::Stdeva
        | Function::Stdevpa
        | Function::StDevPCompat
        | Function::VarS
        | Function::VarP
        | Function::VarA
        | Function::VarpA
        | Function::VarCompat
        | Function::VarPCompat
        | Function::Median
        | Function::Geomean
        | Function::Harmean
        | Function::Avedev
        | Function::Devsq
        | Function::Kurt
        | Function::Skew
        | Function::SkewP => 0,
        Function::Subtotal => 1,
        Function::Index => {
            let Some(Node::OpUnionKind(_)) = args.first() else {
                return None;
            };
            let mut areas = Vec::new();
            flatten_union(&args[0], &mut areas);
            let area = match args.get(3) {
                None => 1,
                Some(Node::NumberKind(n)) => n.floor() as i64,
                Some(_) => return None,
            };
            if area < 1 || area as usize > areas.len() {
                return None;
            }
            let mut out = vec![areas[area as usize - 1].clone()];
            out.extend(args[1..args.len().min(3)].iter().cloned());
            return Some(out);
        }
        _ => return None,
    };
    let mut out = Vec::with_capacity(args.len() + 2);
    for (i, arg) in args.iter().enumerate() {
        if i >= first {
            flatten_union(arg, &mut out);
        } else if matches!(arg, Node::OpUnionKind(_)) {
            return None;
        } else {
            out.push(arg.clone());
        }
    }
    Some(out)
}
