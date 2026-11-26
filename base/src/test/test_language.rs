#![allow(clippy::unwrap_used)]

use crate::{functions::Function, language::get_language, test::util::new_empty_model, Model};

pub fn new_german_empty_model<'a>() -> Model<'a> {
    Model::new_empty("model", "en", "UTC", "de").unwrap()
}

#[test]
fn german() {
    let mut model = new_german_empty_model();
    model._set("A1", "=WENN(1>2, 3, 4)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"4");
}

#[test]
fn french() {
    let mut model = new_empty_model();
    model._set("A1", "=IF(1>2, 3, 4)");
    model._set("B1", "=TRUE");
    model._set("C1", "=FALSE()");
    model._set("D1", "=FALSE");
    model.evaluate();
    model.set_language("fr").unwrap();
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"4");
    assert_eq!(model._get_formula("A1"), *"=SI(1>2,3,4)");
    assert_eq!(model._get_formula("B1"), *"=VRAI");
    assert_eq!(model._get_formula("C1"), *"=FAUX()");
    assert_eq!(model._get_formula("D1"), *"=FAUX");
    assert_eq!(model._get_text("B1"), *"VRAI");
    assert_eq!(model._get_text("C1"), *"FAUX");
    assert_eq!(model._get_text("D1"), *"FAUX");
}

#[test]
fn spanish() {
    let mut model = new_empty_model();
    model._set("A1", "=TRUE()");
    model.evaluate();
    model.set_language("es").unwrap();
    model._set("B1", "=TRUE()");
    model.evaluate();

    assert_eq!(model._get_formula("A1"), *"=VERDADERO()");
    assert_eq!(model._get_text("A1"), *"VERDADERO");
    assert_eq!(model._get_text("B1"), *"#¿NOMBRE?");
}

// Excel's French names, from the syntax section of each function's page on
// support.microsoft.com/fr-fr: MAX.SI.ENS, MIN.SI.ENS, EQUIVX, RANG, CENTILE.
#[test]
fn french_names_match_excel() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("A2", "5");
    model._set("B1", "x");
    model._set("B2", "x");
    model._set("C1", "=MAXIFS(A1:A2,B1:B2,\"x\")");
    model._set("C2", "=MINIFS(A1:A2,B1:B2,\"x\")");
    model._set("C3", "=XMATCH(5,A1:A2)");
    model._set("C4", "=RANK(5,A1:A2)");
    model._set("C5", "=PERCENTILE(A1:A2,0.5)");
    model.evaluate();
    model.set_language("fr").unwrap();

    assert_eq!(model._get_formula("C1"), *"=MAX.SI.ENS(A1:A2,B1:B2,\"x\")");
    assert_eq!(model._get_formula("C2"), *"=MIN.SI.ENS(A1:A2,B1:B2,\"x\")");
    assert_eq!(model._get_formula("C3"), *"=EQUIVX(5,A1:A2)");
    assert_eq!(model._get_formula("C4"), *"=RANG(5,A1:A2)");
    assert_eq!(model._get_formula("C5"), *"=CENTILE(A1:A2,0.5)");
}

#[test]
fn french_names_are_parsed() {
    let mut model = Model::new_empty("model", "fr", "UTC", "fr").unwrap();
    model._set("A1", "3");
    model._set("A2", "5");
    model._set("B1", "x");
    model._set("B2", "x");
    model._set("C1", "=MAX.SI.ENS(A1:A2;B1:B2;\"x\")");
    model._set("C2", "=MIN.SI.ENS(A1:A2;B1:B2;\"x\")");
    model._set("C3", "=EQUIVX(5;A1:A2)");
    model._set("C4", "=RANG(5;A1:A2)");
    model._set("C5", "=CENTILE(A1:A2;0,5)");
    model.evaluate();

    assert_eq!(model._get_text("C1"), *"5");
    assert_eq!(model._get_text("C2"), *"3");
    assert_eq!(model._get_text("C3"), *"2");
    assert_eq!(model._get_text("C4"), *"1");
    assert_eq!(model._get_text("C5"), *"4");
}

// Each function has its own name in every language: otherwise typing that name gives the first
// function that has it (in French, TBILLEQ and YIELDDISC were both TAUX.ESCOMPTE.R).
#[test]
fn every_function_name_reads_back_as_the_same_function() {
    for code in ["en", "es", "fr", "de", "it"] {
        let language = get_language(code).unwrap();
        for function in Function::into_iter() {
            let name = function.to_localized_name(language);
            assert_eq!(
                language.functions.lookup(&name),
                Some(function.clone()),
                "{code}: {name}"
            );
        }
    }
}
