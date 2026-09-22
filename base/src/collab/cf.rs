use crate::collab::bind::Host;
use crate::collab::model::{CollabModel, Stable};
use crate::expressions::parser::stringify::to_english_string;
use crate::expressions::types::CellReferenceRC;
use crate::types::{Position, RangeRef};

/// The top-left of the bounding box of all its areas. An unbounded axis starts at 1.
pub(crate) fn cf_anchor(ranges: &[RangeRef]) -> Option<(i32, i32)> {
    ranges
        .iter()
        .map(|range| {
            let (row, column, ..) = range.resolve();
            (row, column)
        })
        .reduce(|(r, c), (r1, c1)| (r.min(r1), c.min(c1)))
}

impl CollabModel<'_> {
    /// Prints every rule's bound formula slots back into the strings the rule carries, against the
    /// anchor the rule's ranges currently sit at.
    pub(crate) fn normalize_cf(&mut self) {
        for i in 0..self.workbook.worksheets.len() {
            for at in 0..self.workbook.worksheets[i].conditional_formatting.len() {
                let sheet = &self.workbook.worksheets[i];
                let Some(key) = sheet.index.registers.cf_order.get(at) else {
                    continue;
                };
                let Some(bodies) = sheet.index.registers.cf_formulas.get(key).cloned() else {
                    continue;
                };
                // A rule whose ranges all collapsed formats nothing, so there is no anchor to
                // print against and nothing to show.
                let ordinal: Vec<RangeRef> = sheet.conditional_formatting[at]
                    .ranges
                    .iter()
                    .filter_map(|r| {
                        Stable::resolve_range(r, &sheet.index).map(|(r1, c1, r2, c2)| RangeRef {
                            rows: Some((r1, r2)),
                            cols: Some((c1, c2)),
                        })
                    })
                    .collect();
                let Some((row, column)) = cf_anchor(&ordinal) else {
                    continue;
                };
                let context = CellReferenceRC {
                    sheet: sheet.get_name(),
                    row,
                    column,
                };
                let host = Host::relative(i as u32, row, column);
                let texts: Vec<String> = bodies
                    .iter()
                    .map(|body| match self.lower(&body.formula, &host) {
                        Ok(node) => {
                            let text = to_english_string(&node, &context);
                            match body.equals {
                                true => format!("={text}"),
                                false => text,
                            }
                        }
                        Err(_) => "#REF!".to_string(),
                    })
                    .collect();
                let rule = &mut self.workbook.worksheets[i].conditional_formatting[at].cf_rule;
                for (slot, text) in rule.formulas_mut().into_iter().zip(texts) {
                    *slot = text;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use crate::cf_types::{
        CfRule, CfRuleInput, Cfvo, ColorScaleThreshold, ConditionalFormattingView, ValueOperator,
    };
    use crate::collab::model::{CollabModel, Stable};
    use crate::collab::spill::test::{converged, deliver, peer, Peer};
    use crate::types::{Color, Dxf, Fill};
    use crate::UserModel;

    fn fill(color: &str) -> Dxf {
        Dxf {
            fill: Some(Fill {
                color: Color::Rgb(color.to_string()),
            }),
            ..Default::default()
        }
    }

    fn cell_is_gt(threshold: &str, format: Dxf) -> CfRuleInput {
        CfRuleInput::CellIs {
            operator: ValueOperator::GreaterThan,
            formula: threshold.to_string(),
            formula2: None,
            format,
            stop_if_true: false,
        }
    }

    fn formula_rule(formula: &str, format: Dxf) -> CfRuleInput {
        CfRuleInput::Formula {
            formula: formula.to_string(),
            format,
            stop_if_true: false,
        }
    }

    fn color_scale(formula: &str) -> CfRuleInput {
        CfRuleInput::ColorScale {
            thresholds: vec![
                ColorScaleThreshold {
                    cfvo: Cfvo::Formula(formula.to_string()),
                    color: Color::Rgb("#FFFFFF".to_string()),
                },
                ColorScaleThreshold {
                    cfvo: Cfvo::Max,
                    color: Color::Rgb("#000000".to_string()),
                },
            ],
        }
    }

    fn list(model: &Peer) -> Vec<ConditionalFormattingView> {
        model.get_conditional_formatting_list(0).unwrap()
    }

    fn color_at(model: &Peer, row: i32, column: i32) -> Color {
        model
            .get_extended_cell_style(0, row, column)
            .unwrap()
            .style
            .fill
            .color
    }

    /// The list as a reader sees it, with the replica-local `dxf_id` replaced by the format it
    /// names: two replicas agree on what is shown, not on how they index their own tables.
    fn shown(model: &Peer, sheet: u32) -> Vec<(usize, String, CfRule, u32, Option<Dxf>)> {
        model
            .get_conditional_formatting_list(sheet)
            .unwrap()
            .into_iter()
            .map(|mut view| {
                let dxf = model
                    .get_dxf_for_conditional_formatting(sheet, view.index as u32)
                    .unwrap();
                if let Some(slot) = view.cf_rule.dxf_id_mut() {
                    *slot = 0;
                }
                (view.index, view.range, view.cf_rule, view.priority, dxf)
            })
            .collect()
    }

    fn dxfs(model: &Peer) -> Vec<Dxf> {
        model.get_model().workbook.styles.dxfs.clone()
    }

    fn assert_cf(a: &Peer, b: &Peer) {
        converged(a, b); // same cell contents
        assert_eq!(shown(a, 0), shown(b, 0), "conditional formatting list");
        for row in 1..=6 {
            for column in 1..=4 {
                // same rendering
                assert_eq!(
                    color_at(a, row, column),
                    color_at(b, row, column),
                    "fill at ({row}, {column})"
                );
            }
        }
    }

    fn sync(a: &mut Peer, b: &mut Peer) {
        let (from_a, from_b) = (a.flush_send_queue(), b.flush_send_queue());
        b.apply_external_diffs(&from_a).unwrap();
        a.apply_external_diffs(&from_b).unwrap();
    }

    #[test]
    fn dxf_travels_by_value() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        a.set_user_input(0, 1, 1, "5").unwrap(); // A1=5
        deliver(&mut a, &mut b);

        // B's dxf table starts out different: a rule it added and deleted left its format behind.
        b.add_conditional_formatting(0, "A1:A5", cell_is_gt("0", fill("#00FF00")))
            .unwrap();
        b.delete_conditional_formatting(0, 0).unwrap();
        a.add_conditional_formatting(0, "A1:A5", cell_is_gt("0", fill("#FF0000")))
            .unwrap();
        sync(&mut a, &mut b);

        assert_ne!(dxfs(&a), dxfs(&b), "the two tables are ordered differently");
        for model in [&a, &b] {
            assert_eq!(color_at(model, 1, 1), Color::Rgb("#FF0000".to_string()));
            assert_eq!(
                model.get_dxf_for_conditional_formatting(0, 0).unwrap(),
                Some(fill("#FF0000"))
            );
        }
        assert_cf(&a, &b);

        a.update_conditional_formatting(0, 0, "A1:A5", cell_is_gt("0", fill("#0000FF")))
            .unwrap();
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(color_at(model, 1, 1), Color::Rgb("#0000FF".to_string()));
        }
        assert_cf(&a, &b);

        let (before_a, before_b) = (dxfs(&a).len(), dxfs(&b).len());
        for _ in 0..3 {
            a.undo().unwrap();
            a.redo().unwrap();
        }
        deliver(&mut a, &mut b);
        assert_eq!(dxfs(&a).len(), before_a, "A's dxf table grew");
        assert_eq!(dxfs(&b).len(), before_b, "B's dxf table grew");
        assert_eq!(color_at(&a, 1, 1), Color::Rgb("#0000FF".to_string()));
        assert_cf(&a, &b);
    }

    #[test]
    fn formulas_follow_structure() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        for row in 1..=5 {
            a.set_user_input(0, row, 1, &row.to_string()).unwrap();
        }
        a.set_user_input(0, 1, 4, "7").unwrap();
        deliver(&mut a, &mut b);

        let rules: Vec<(&str, CfRuleInput)> = vec![
            ("A1:A5", formula_rule("=$D$1>0", fill("#FF0000"))),
            ("A1:A5", formula_rule("=A1>2", fill("#0000FF"))),
            ("A1:A5", cell_is_gt("=$D$1", fill("#00FF00"))),
            ("A1:A5", color_scale("=$D$1")),
        ];
        let mut oracle = UserModel::new_empty("model", "en", "UTC", "en").unwrap();
        for row in 1..=5 {
            oracle.set_user_input(0, row, 1, &row.to_string()).unwrap();
        }
        oracle.set_user_input(0, 1, 4, "7").unwrap();
        for (range, rule) in &rules {
            a.add_conditional_formatting(0, range, rule.clone())
                .unwrap();
            oracle
                .add_conditional_formatting(0, range, rule.clone())
                .unwrap();
        }
        // B has not seen the rules; its structural edit still has to move them.
        b.insert_rows(0, 1, 1).unwrap();
        b.insert_columns(0, 1, 1).unwrap();
        sync(&mut a, &mut b);
        oracle.insert_rows(0, 1, 1).unwrap();
        oracle.insert_columns(0, 1, 1).unwrap();

        assert_cf(&a, &b);
        assert_eq!(list(&a), oracle.get_conditional_formatting_list(0).unwrap());
        for row in 1..=6 {
            for column in 1..=4 {
                assert_eq!(
                    a.get_extended_cell_style(0, row, column)
                        .unwrap()
                        .style
                        .fill
                        .color,
                    oracle
                        .get_extended_cell_style(0, row, column)
                        .unwrap()
                        .style
                        .fill
                        .color,
                    "fill at ({row}, {column})"
                );
            }
        }
    }

    #[test]
    fn rename_shows_in_cf_formula() {
        let mut a = peer(1);
        a.new_sheet().unwrap();
        a.new_sheet().unwrap();
        a.rename_sheet(1, "Data").unwrap();
        a.set_user_input(1, 1, 1, "9").unwrap();
        a.add_conditional_formatting(0, "A1:A5", formula_rule("=Data!A1>0", fill("#FF0000")))
            .unwrap();
        assert_eq!(list(&a)[0].cf_rule, {
            formula_rule("=Data!A1>0", fill("#FF0000")).split().0
        });
        a.rename_sheet(1, "Info").unwrap();
        let (mut expected, _) = formula_rule("=Info!A1>0", fill("#FF0000")).split();
        if let Some(slot) = expected.dxf_id_mut() {
            *slot = 0;
        }
        assert_eq!(list(&a)[0].cf_rule, expected);

        // A deleted corner row clamps: the list shows the range evaluation still formats.
        a.delete_rows(0, 1, 1).unwrap();
        assert_eq!(list(&a)[0].range, "A1:A4");
        // The reference is relative, so the row now first still looks at `Info!A1`.
        assert_eq!(color_at(&a, 1, 1), Color::Rgb("#FF0000".to_string()));
    }

    #[test]
    fn priority_moves_converge() {
        // The rule each list entry shows, by the dxf its storage index resolves to.
        fn order(model: &Peer) -> Vec<Option<Dxf>> {
            list(model)
                .iter()
                .map(|view| {
                    model
                        .get_dxf_for_conditional_formatting(0, view.index as u32)
                        .unwrap()
                })
                .collect()
        }

        fn setup(count: usize) -> (Peer, Peer) {
            let (mut a, mut b) = (peer(1), peer(2));
            a.new_sheet().unwrap();
            a.set_user_input(0, 1, 1, "5").unwrap();
            for at in 0..count {
                let color = format!("#0000{:02X}", at + 1);
                a.add_conditional_formatting(0, "A1:A5", cell_is_gt("0", fill(&color)))
                    .unwrap();
            }
            deliver(&mut a, &mut b);
            (a, b)
        }

        // (a) both peers raise the same rule.
        {
            let (mut a, mut b) = setup(3);
            a.raise_conditional_formatting_priority(0, 0).unwrap();
            b.raise_conditional_formatting_priority(0, 0).unwrap();
            sync(&mut a, &mut b);
            assert_cf(&a, &b);
            assert_eq!(order(&a), order(&b));
        }

        // (b) two moves into the same gap, then another move across the tie.
        {
            let (mut a, mut b) = setup(4);
            a.raise_conditional_formatting_priority(0, 0).unwrap(); // A over B
            b.lower_conditional_formatting_priority(0, 3).unwrap(); // D under C
            sync(&mut a, &mut b);
            assert_cf(&a, &b);
            // Both minted into the gap between B and C: same position, different session.
            let registers = &a.get_model().workbook.worksheets[0].index.registers;
            let moved = |at: usize| &registers.cf_positions[&registers.cf_order[at]].value;
            assert_eq!(moved(0).position(), moved(3).position());
            assert_ne!(moved(0), moved(3));
            let priority = |m: &Peer| list(m).iter().find(|v| v.index == 0).unwrap().priority;
            assert_eq!(priority(&a), 2);
            a.raise_conditional_formatting_priority(0, 0).unwrap();
            deliver(&mut a, &mut b);
            assert_cf(&a, &b);
            assert_eq!(priority(&a), 3, "A stepped over the tie, below C");
            assert_eq!(order(&a), order(&b));
        }

        // (c) one raises A over B while the other lowers B under A: one swap, not two.
        {
            let (mut a, mut b) = setup(2);
            let before = order(&a);
            a.raise_conditional_formatting_priority(0, 0).unwrap();
            b.lower_conditional_formatting_priority(0, 1).unwrap();
            sync(&mut a, &mut b);
            assert_cf(&a, &b);
            let after = order(&a);
            assert_ne!(after, before, "the rules did swap");
            assert_eq!(after, vec![before[1].clone(), before[0].clone()]);
        }

        // (d) a raise racing a delete: the rule stays deleted, and comes back where it was raised
        // to once the delete is undone.
        {
            let (mut a, mut b) = setup(3);
            a.raise_conditional_formatting_priority(0, 0).unwrap();
            b.delete_conditional_formatting(0, 0).unwrap();
            sync(&mut a, &mut b);
            assert_cf(&a, &b);
            assert_eq!(list(&a).len(), 2);
            b.undo().unwrap();
            deliver(&mut b, &mut a);
            assert_cf(&a, &b);
            assert_eq!(list(&a).len(), 3);
            // The raised rule is back above its old neighbour: it is the middle entry.
            assert_eq!(order(&a)[1], Some(fill("#000001")));
        }

        // (e) a raise to the top racing a fresh rule.
        {
            let (mut a, mut b) = setup(2);
            a.raise_conditional_formatting_priority(0, 0).unwrap();
            b.add_conditional_formatting(0, "A1:A5", cell_is_gt("0", fill("#00FF00")))
                .unwrap();
            sync(&mut a, &mut b);
            assert_cf(&a, &b);
            assert_eq!(order(&a), order(&b));
        }

        // (f) undoing a raise after the peer raised as well.
        {
            let (mut a, mut b) = setup(3);
            a.raise_conditional_formatting_priority(0, 0).unwrap();
            b.raise_conditional_formatting_priority(0, 2).unwrap();
            sync(&mut a, &mut b);
            a.undo().unwrap();
            deliver(&mut a, &mut b);
            assert_cf(&a, &b);
            assert_eq!(order(&a), order(&b));
        }

        // A move never moves a storage index: the index a list entry carries still resolves to the
        // rule it was taken from.
        let (mut a, _) = setup(3);
        let before: Vec<Option<Dxf>> = (0..3)
            .map(|at| a.get_dxf_for_conditional_formatting(0, at).unwrap())
            .collect();
        a.raise_conditional_formatting_priority(0, 0).unwrap();
        a.lower_conditional_formatting_priority(0, 2).unwrap();
        let after: Vec<Option<Dxf>> = (0..3)
            .map(|at| a.get_dxf_for_conditional_formatting(0, at).unwrap())
            .collect();
        assert_eq!(before, after);

        // A key is minted past every rule the sheet ever held, so a deleted one cannot be revived
        // by the next add landing on its key.
        let (mut a, _) = setup(3);
        a.delete_conditional_formatting(0, 0).unwrap();
        a.add_conditional_formatting(0, "A1:A5", cell_is_gt("0", fill("#00FF00")))
            .unwrap();
        assert_eq!(list(&a).len(), 3);
    }

    /// A rule, its format and a moved priority all survive a sheet copy, an undone sheet delete
    /// and a snapshot round trip.
    #[test]
    fn carry_over() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        a.set_user_input(0, 1, 1, "5").unwrap();
        a.add_conditional_formatting(0, "A1:A5", cell_is_gt("0", fill("#FF0000")))
            .unwrap();
        a.add_conditional_formatting(0, "A1:A5", formula_rule("=A1>0", fill("#00FF00")))
            .unwrap();
        // The first rule now outranks the second, which is not the storage order.
        a.raise_conditional_formatting_priority(0, 0).unwrap();
        deliver(&mut a, &mut b);
        assert_cf(&a, &b);

        a.duplicate_sheet(0).unwrap();
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            let copy = model.get_conditional_formatting_list(1).unwrap();
            assert_eq!(copy.len(), 2);
            assert_eq!(copy[0].index, 0, "the raised rule still leads");
            assert_eq!(copy[0].cf_rule, list(model)[0].cf_rule);
            assert_eq!(
                model.get_dxf_for_conditional_formatting(1, 0).unwrap(),
                Some(fill("#FF0000"))
            );
            assert_eq!(copy[1].range, "A1:A5");
        }

        a.delete_sheet(1).unwrap();
        a.undo().unwrap();
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            let copy = model.get_conditional_formatting_list(1).unwrap();
            assert_eq!(copy.len(), 2);
            assert_eq!(copy[0].index, 0);
            assert_eq!(
                model.get_dxf_for_conditional_formatting(1, 0).unwrap(),
                Some(fill("#FF0000"))
            );
        }

        let mut restored = UserModel::<Stable>::from_bytes_with_session(&a.to_bytes(), 3).unwrap();
        restored.evaluate();
        assert_eq!(
            restored.get_conditional_formatting_list(0).unwrap(),
            list(&a)
        );
        assert_eq!(
            restored.get_dxf_for_conditional_formatting(0, 0).unwrap(),
            Some(fill("#FF0000"))
        );
        assert_eq!(
            restored
                .get_extended_cell_style(0, 1, 1)
                .unwrap()
                .style
                .fill
                .color,
            a.get_extended_cell_style(0, 1, 1).unwrap().style.fill.color
        );
    }

    /// Function calls under different language should be localized.
    #[test]
    fn localise_function_names_in_peers_lang() {
        // Peer A uses Spanish as a language
        let mut a = UserModel::<Stable>::new_empty_with_session("a", "en", "UTC", "es", 1).unwrap();
        let mut b = peer(2);
        a.set_user_input(0, 1, 1, "5").unwrap();
        a.set_user_input(0, 1, 2, "=SUM(A1,1)").unwrap();
        a.add_conditional_formatting(0, "A1:A5", formula_rule("=SUM($A$1,0)>0", fill("#FF0000")))
            .unwrap();
        a.new_defined_name("plus1", None, "=LAMBDA(x,SUM(x,1))")
            .unwrap();
        a.set_user_input(0, 2, 2, "=plus1(A1)").unwrap();
        deliver(&mut a, &mut b);

        assert_eq!(a.get_cell_content(0, 1, 2).unwrap(), "=SUMA(A1,1)"); // lang=es
        assert_eq!(b.get_cell_content(0, 1, 2).unwrap(), "=SUM(A1,1)"); // lang=en
        for m in [&a, &b] {
            assert_eq!(m.get_formatted_cell_value(0, 1, 2).unwrap(), "6");
            assert_eq!(m.get_formatted_cell_value(0, 2, 2).unwrap(), "6");
            assert_eq!(color_at(m, 1, 1), Color::Rgb("#FF0000".to_string()));
        }
        assert!(matches!(
            &list(&a)[0].cf_rule,
            CfRule::Formula { formula, .. } if formula == "=SUMA($A$1,0)>0"
        ));
        assert!(matches!(
            &list(&b)[0].cf_rule,
            CfRule::Formula { formula, .. } if formula == "=SUM($A$1,0)>0"
        ));
    }

    #[test]
    fn set_language_is_local_and_relocalizes_reads() {
        let mut a = UserModel::<Stable>::new_empty_with_session("a", "en", "UTC", "en", 1).unwrap();
        let mut b = peer(2);
        a.set_user_input(0, 1, 1, "5").unwrap();
        a.set_user_input(0, 1, 2, "=SUM(A1,1)").unwrap();
        a.add_conditional_formatting(0, "A1:A5", formula_rule("=SUM($A$1,0)>0", fill("#FF0000")))
            .unwrap();
        a.new_defined_name("plus1", None, "=LAMBDA(x,SUM(x,1))")
            .unwrap();
        deliver(&mut a, &mut b);

        a.set_language("es").unwrap();
        b.set_language("fr").unwrap();

        assert_eq!(a.get_cell_content(0, 1, 2).unwrap(), "=SUMA(A1,1)");
        assert_eq!(b.get_cell_content(0, 1, 2).unwrap(), "=SOMME(A1,1)");
        assert!(matches!(
            &list(&a)[0].cf_rule,
            CfRule::Formula { formula, .. } if formula == "=SUMA($A$1,0)>0"
        ));
        assert!(matches!(
            &list(&b)[0].cf_rule,
            CfRule::Formula { formula, .. } if formula == "=SOMME($A$1,0)>0"
        ));
        assert_eq!(a.get_defined_name_list()[0].2, "=LAMBDA(x,SUMA(x,1))");
        assert_eq!(b.get_defined_name_list()[0].2, "=LAMBDA(x,SOMME(x,1))");
        for m in [&a, &b] {
            assert_eq!(m.get_formatted_cell_value(0, 1, 2).unwrap(), "6");
            assert_eq!(color_at(m, 1, 1), Color::Rgb("#FF0000".to_string()));
        }
        // Nothing was replicated: the switch is a local preference, not an edit.
        assert!(a.model.flush().is_empty());
        assert!(b.model.flush().is_empty());

        // Input is parsed under the new language too, and still travels in English.
        a.set_user_input(0, 1, 3, "=SUMA(A1,2)").unwrap();
        deliver(&mut a, &mut b);
        assert_eq!(a.get_cell_content(0, 1, 3).unwrap(), "=SUMA(A1,2)");
        assert_eq!(b.get_cell_content(0, 1, 3).unwrap(), "=SOMME(A1,2)");
        for m in [&a, &b] {
            assert_eq!(m.get_formatted_cell_value(0, 1, 3).unwrap(), "7");
        }

        // An unknown language is rejected and leaves the model on the one it had.
        let error = a.set_language("xx").unwrap_err();
        assert!(error.contains("Invalid language"), "{error}");
        assert_eq!(a.get_language(), "es");
    }

    /// A rule minted by the import path reads back the way the ordinal model shows it.
    #[test]
    fn import_keeps_rules() {
        let mut source = crate::Model::new_empty("import", "en", "UTC", "en").unwrap();
        source.set_user_input(0, 1, 1, "5".to_string()).unwrap();
        source
            .add_conditional_formatting(0, "A1:A5", formula_rule("=A1>0", fill("#FF0000")))
            .unwrap();
        source
            .add_conditional_formatting(0, "A1:A5", cell_is_gt("1", fill("#0000FF")))
            .unwrap();
        source
            .add_conditional_formatting(
                0,
                "A1:A5",
                formula_rule("=SUM($A$1:$A$2)>0", fill("#00FF00")),
            )
            .unwrap();
        source
            .new_defined_name("plus1", None, "=LAMBDA(x,SUM(x,1))")
            .unwrap();
        source
            .set_user_input(0, 2, 2, "=plus1(A1)".to_string())
            .unwrap();
        // Storage order and priority order disagree, which the import has to carry over.
        source.raise_conditional_formatting_priority(0, 0).unwrap();
        source.evaluate();

        // Under Spanish, so a stored English name has to bind as the builtin it is, not as a
        // named function the Spanish parser does not know.
        let mut imported =
            CollabModel::from_workbook_with_session(source.workbook.clone(), "es", 1).unwrap();
        imported.evaluate();

        fn stored(m: &CollabModel<'_>) -> Vec<CfRule> {
            m.workbook.worksheets[0]
                .conditional_formatting
                .iter()
                .map(|cf| cf.cf_rule.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            stored(&imported).len(),
            source.workbook.worksheets[0].conditional_formatting.len()
        );
        assert!(stored(&imported).iter().any(
            |r| matches!(r, CfRule::Formula { formula, .. } if formula == "=SUM($A$1:$A$2)>0")
        ));
        assert!(imported
            .get_conditional_formatting_list(0)
            .unwrap()
            .iter()
            .any(|v| matches!(&v.cf_rule, CfRule::Formula { formula, .. } if formula == "=SUMA($A$1:$A$2)>0")));
        assert_eq!(
            imported.get_defined_name_list()[0].2,
            "=LAMBDA(x,SUMA(x,1))"
        );
        assert_eq!(imported.get_formatted_cell_value(0, 2, 2).unwrap(), "6");
        assert_eq!(
            imported
                .get_extended_style_for_cell(0, 1, 1)
                .unwrap()
                .style
                .fill
                .color,
            source
                .get_extended_style_for_cell(0, 1, 1)
                .unwrap()
                .style
                .fill
                .color
        );

        let mut imported =
            CollabModel::from_workbook_with_session(source.workbook.clone(), "en", 1).unwrap();
        imported.evaluate();
        assert_eq!(
            imported.get_conditional_formatting_list(0).unwrap(),
            source.get_conditional_formatting_list(0).unwrap()
        );
        assert_eq!(
            imported
                .get_extended_style_for_cell(0, 1, 1)
                .unwrap()
                .style
                .fill
                .color,
            source
                .get_extended_style_for_cell(0, 1, 1)
                .unwrap()
                .style
                .fill
                .color
        );
    }
}
