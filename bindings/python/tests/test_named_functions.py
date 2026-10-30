import ironcalc as ic
import pytest


def profit_parameters():
    return [
        {"name": "revenue", "description": "Total revenue"},
        {"name": "cost"},
    ]


def test_new_named_function_and_call(um):
    um.new_named_function(
        "PROFIT",
        "(revenue - cost) / revenue",
        parameters=profit_parameters(),
        description="Profit margin",
    )
    um.set_user_input(0, 1, 1, "=PROFIT(200, 50)")
    assert um.get_formatted_cell_value(0, 1, 1) == "0.75"
    assert um.get_named_function_list() == [
        {
            "name": "PROFIT",
            "scope": None,
            "description": "Profit margin",
            "parameters": [
                {"name": "revenue", "description": "Total revenue", "optional": False},
                {"name": "cost", "description": "", "optional": False},
            ],
            "formula": "(revenue-cost)/revenue",
        }
    ]
    assert um.get_defined_name_list() == []


def test_lambda_defined_name_is_a_named_function(um):
    um.new_defined_name("Twice", "=LAMBDA(x, 2*x)", comment="Doubles")
    assert um.get_defined_name_list() == []
    functions = um.get_named_function_list()
    assert len(functions) == 1
    assert functions[0]["name"] == "Twice"
    assert functions[0]["description"] == "Doubles"
    assert functions[0]["parameters"] == [
        {"name": "x", "description": "", "optional": False}
    ]
    assert functions[0]["formula"] == "2*x"


def test_update_and_delete(um):
    um.new_named_function("DOUBLE", "x*2", parameters=[{"name": "x"}])
    um.set_user_input(0, 1, 1, "=DOUBLE(4)")
    um.update_named_function(
        "DOUBLE", None, "TWICE", "x*2", new_parameters=[{"name": "x"}], new_scope=0
    )
    functions = um.get_named_function_list()
    assert functions[0]["name"] == "TWICE"
    assert functions[0]["scope"] == 0
    # Renamed in the cells that use it
    assert um.get_cell_content(0, 1, 1) == "=TWICE(4)"
    assert um.get_formatted_cell_value(0, 1, 1) == "8"
    assert um.named_function_usage_count("TWICE", 0) == 1

    um.undo()
    assert um.get_named_function_list()[0]["name"] == "DOUBLE"
    assert um.get_cell_content(0, 1, 1) == "=DOUBLE(4)"

    um.delete_named_function("DOUBLE", None)
    assert um.get_named_function_list() == []
    assert um.get_formatted_cell_value(0, 1, 1) == "#NAME?"
    um.undo()
    assert um.get_formatted_cell_value(0, 1, 1) == "8"


def test_errors(um):
    with pytest.raises(BaseException, match="SUM is already a built-in function"):
        um.new_named_function("SUM", "1")
    um.new_named_function("Fn", "1")
    with pytest.raises(BaseException, match="already exists"):
        um.new_defined_name("fn", "Sheet1!$A$1")
    with pytest.raises(BaseException, match="Named function not found"):
        um.delete_named_function("Nope", None)


def test_check(um):
    um.new_named_function("DOUBLE", "x*2", parameters=[{"name": "x"}])
    check = um.check_named_function(
        "double",
        "(revenue - coste) / price",
        parameters=[{"name": "revenue"}, {"name": "cost"}, {"name": "B2"}],
    )
    assert check == {
        "name": "A function or defined name called DOUBLE already exists.",
        "parameters": [
            None,
            None,
            "This name looks like a cell reference. Try another.",
        ],
        "formula": [
            '"coste" isn\'t an argument. Add it above or check the spelling.',
            '"price" isn\'t an argument. Add it above or check the spelling.',
        ],
        "warnings": [
            '"cost" isn\'t used in the formula.',
            '"B2" isn\'t used in the formula.',
        ],
        "unknown_names": ["coste", "price"],
    }
    # Editing keeps its own name
    check = um.check_named_function(
        "double", "x*3", parameters=[{"name": "x"}], current_name="DOUBLE"
    )
    assert check == {
        "name": None,
        "parameters": [None],
        "formula": [],
        "warnings": [],
        "unknown_names": [],
    }


def test_raw_model(rm):
    rm.new_named_function("DOUBLE", "x*2", parameters=[{"name": "x"}])
    rm.set_user_input(0, 1, 1, "=DOUBLE(21)")
    rm.evaluate()
    assert rm.get_formatted_cell_value(0, 1, 1) == "42"
    assert rm.named_function_usage_count("DOUBLE", None) == 1
    rm.update_named_function("DOUBLE", None, "TWICE", "x*2", new_parameters=[{"name": "x"}])
    assert rm.get_named_function_list()[0]["name"] == "TWICE"
    rm.delete_named_function("TWICE", None)
    assert rm.get_named_function_list() == []
