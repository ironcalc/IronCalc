"""IronCalc: create, edit and evaluate Excel spreadsheets.

There are two APIs:

* The **user API** (:class:`UserModel`): the same high level API used by the
  IronCalc web application. Every action evaluates the workbook, keeps
  undo/redo history and produces diffs for collaboration.
* The **raw API** (:class:`Model`): a low level API. Nothing is evaluated
  automatically (call :meth:`Model.evaluate` yourself), there is no undo/redo
  and no diffs. It is faster and more flexible, but easier to get wrong.

Quick start::

    import ironcalc as ic

    model = ic.UserModel("my-workbook")
    model.set_user_input(0, 1, 1, "=1+2")
    print(model.get_formatted_cell_value(0, 1, 1))  # "3"
    model.save_to_xlsx("my-workbook.xlsx")
"""

from ironcalc._ironcalc import (
    CellType,
    Model,
    UserModel,
    WorkbookError,
    __version__,
    column_name_from_number,
    column_number_from_name,
    create,
    create_user_model,
    create_user_model_from_bytes,
    create_user_model_from_icalc,
    create_user_model_from_xlsx,
    get_all_timezones,
    get_supported_locales,
    load_from_bytes,
    load_from_icalc,
    load_from_xlsx,
    quote_name,
    test_panic,
)

__all__ = [
    "CellType",
    "Model",
    "UserModel",
    "WorkbookError",
    "__version__",
    "column_name_from_number",
    "column_number_from_name",
    "create",
    "create_user_model",
    "create_user_model_from_bytes",
    "create_user_model_from_icalc",
    "create_user_model_from_xlsx",
    "get_all_timezones",
    "get_supported_locales",
    "load_from_bytes",
    "load_from_icalc",
    "load_from_xlsx",
    "quote_name",
    "test_panic",
]
