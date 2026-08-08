Raw API Reference
-----------------

The raw API (:class:`Model`) is a low level API. Nothing is evaluated
automatically: call :meth:`Model.evaluate` after making changes, otherwise the
workbook may be in an inconsistent state. There is no undo/redo history and no
diffs are produced. It is faster and more flexible than the user API, but
easier to get wrong.

Conventions are the same as in the user API: 0-based ``sheet`` indexes,
1-based ``row`` and ``column`` indexes, dictionaries for complex values and
:class:`WorkbookError` on invalid input.

Evaluation and persistence
^^^^^^^^^^^^^^^^^^^^^^^^^^

.. method:: Model.evaluate()

   Evaluates the workbook. Call this after each batch of changes.

.. method:: Model.save_to_xlsx(file: str)
.. method:: Model.save_to_icalc(file: str)
.. method:: Model.to_bytes() -> bytes

Setting values
^^^^^^^^^^^^^^

.. method:: Model.set_user_input(sheet: int, row: int, column: int, value: str)

   Sets the input of a cell as a user would type it: ``"3.5"`` is a number,
   ``"Hello"`` a string, ``"=A1*2"`` a formula.

.. method:: Model.update_cell_with_text(sheet: int, row: int, column: int, value: str)

   Sets a string value without input parsing (``"123"`` stays a string).

.. method:: Model.update_cell_with_number(sheet: int, row: int, column: int, value: float)
.. method:: Model.update_cell_with_bool(sheet: int, row: int, column: int, value: bool)
.. method:: Model.update_cell_with_formula(sheet: int, row: int, column: int, formula: str)
.. method:: Model.set_user_array_formula(sheet, row, column, width, height, formula: str)

   Sets an array (spill) formula covering ``width`` x ``height`` cells.

.. method:: Model.clear_cell_contents(sheet: int, row: int, column: int)

   Clears the content of a single cell, keeping the formatting.

.. method:: Model.range_clear_contents(sheet, start_row, start_column, end_row, end_column)
.. method:: Model.range_clear_all(sheet, start_row, start_column, end_row, end_column)

Getting values
^^^^^^^^^^^^^^

.. method:: Model.get_cell_value(sheet: int, row: int, column: int)

   Returns the value of a cell as a native Python value: ``None``, ``str``,
   ``float`` or ``bool``.

.. method:: Model.get_cell_value_by_ref(cell_ref: str)

   Same, with a reference like ``"Sheet1!C4"``.

.. method:: Model.get_formatted_cell_value(sheet: int, row: int, column: int) -> str

   Returns the value formatted with the cell's number format (i.e. ``"$5.75"``).

.. method:: Model.get_cell_content(sheet: int, row: int, column: int) -> str

   Returns the content as seen in the editor: the formula if there is one, the
   raw value otherwise.

.. method:: Model.get_cell_formula(sheet: int, row: int, column: int) -> str | None
.. method:: Model.get_cell_type(sheet: int, row: int, column: int) -> CellType
.. method:: Model.is_empty_cell(sheet: int, row: int, column: int) -> bool
.. method:: Model.get_all_cells() -> list[tuple[int, int, int]]

   Returns all non-empty cells as ``(sheet, row, column)`` tuples.

.. method:: Model.get_sheet_dimensions(sheet: int) -> (int, int, int, int)

   Returns ``(min_row, max_row, min_column, max_column)`` of the non-empty
   cells; ``(1, 1, 1, 1)`` for an empty sheet.

.. method:: Model.get_sheet_markup(sheet: int) -> str

   Returns a markdown-like table of the sheet (formulas, not values), useful
   for debugging and tests.

Styles
^^^^^^

Styles are dictionaries; see :doc:`objects`.

.. method:: Model.get_cell_style(sheet: int, row: int, column: int) -> dict
.. method:: Model.set_cell_style(sheet: int, row: int, column: int, style: dict)
.. method:: Model.get_column_style(sheet: int, column: int) -> dict | None
.. method:: Model.set_column_style(sheet: int, column: int, style: dict)
.. method:: Model.delete_column_style(sheet: int, column: int)
.. method:: Model.get_row_style(sheet: int, row: int) -> dict | None
.. method:: Model.set_row_style(sheet: int, row: int, style: dict)
.. method:: Model.delete_row_style(sheet: int, row: int)

Rows and columns
^^^^^^^^^^^^^^^^

.. method:: Model.insert_rows(sheet: int, row: int, row_count: int)
.. method:: Model.insert_columns(sheet: int, column: int, column_count: int)
.. method:: Model.delete_rows(sheet: int, row: int, row_count: int)
.. method:: Model.delete_columns(sheet: int, column: int, column_count: int)
.. method:: Model.get_column_width(sheet: int, column: int) -> float
.. method:: Model.get_row_height(sheet: int, row: int) -> float
.. method:: Model.set_column_width(sheet: int, column: int, width: float)
.. method:: Model.set_row_height(sheet: int, row: int, height: float)
.. method:: Model.set_column_hidden(sheet: int, column: int, hidden: bool)
.. method:: Model.set_row_hidden(sheet: int, row: int, hidden: bool)
.. method:: Model.is_column_hidden(sheet: int, column: int) -> bool
.. method:: Model.is_row_hidden(sheet: int, row: int) -> bool
.. method:: Model.get_frozen_rows_count(sheet: int) -> int
.. method:: Model.get_frozen_columns_count(sheet: int) -> int
.. method:: Model.set_frozen_rows_count(sheet: int, row_count: int)
.. method:: Model.set_frozen_columns_count(sheet: int, column_count: int)

Sheets
^^^^^^

.. method:: Model.add_sheet(sheet_name: str)
.. method:: Model.new_sheet()

   Adds a sheet with an automatically generated name.

.. method:: Model.delete_sheet(sheet: int)
.. method:: Model.rename_sheet(sheet: int, new_name: str)
.. method:: Model.set_sheet_color(sheet: int, color)

   ``color`` is ``None``, ``"#RRGGBB"`` or ``[theme_index, tint]``.

.. method:: Model.set_sheet_state(sheet: int, state: str)

   ``state`` is ``"visible"``, ``"hidden"`` or ``"veryHidden"``.

.. method:: Model.get_worksheets_properties() -> list[dict]
.. method:: Model.set_show_grid_lines(sheet: int, show_grid_lines: bool)

Defined names
^^^^^^^^^^^^^

.. method:: Model.get_defined_name_list() -> list[dict]
.. method:: Model.new_defined_name(name: str, scope: int | None, formula: str)
.. method:: Model.update_defined_name(name, scope, new_name, new_scope, new_formula)
.. method:: Model.delete_defined_name(name: str, scope: int | None)

Workbook properties
^^^^^^^^^^^^^^^^^^^

.. method:: Model.get_theme() -> dict
.. method:: Model.set_theme(theme: dict)
.. method:: Model.get_timezone() -> str
.. method:: Model.set_timezone(timezone: str)
.. method:: Model.get_locale() -> str
.. method:: Model.set_locale(locale: str)
.. method:: Model.get_language() -> str
.. method:: Model.set_language(language: str)
.. method:: Model.get_fmt_settings() -> dict
