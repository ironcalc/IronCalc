Raw API Reference
-----------------

The raw API (:class:`Model`) is a low level API. Nothing is evaluated
automatically: call :meth:`evaluate` after making changes, otherwise the
workbook may be in an inconsistent state. There is no undo/redo history and no
diffs are produced. It is faster and more flexible than the user API, but
easier to get wrong.

Conventions are the same as in the user API: 0-based ``sheet`` indexes,
1-based ``row`` and ``column`` indexes, dictionaries for complex values and
:class:`WorkbookError` on invalid input.

Evaluation and persistence
^^^^^^^^^^^^^^^^^^^^^^^^^^

.. method:: evaluate()

   Evaluates the workbook. Call this after each batch of changes.

.. method:: save_to_xlsx(file: str)
.. method:: save_to_icalc(file: str)
.. method:: to_bytes() -> bytes

Setting values
^^^^^^^^^^^^^^

.. method:: set_user_input(sheet: int, row: int, column: int, value: str)

   Sets the input of a cell as a user would type it: ``"3.5"`` is a number,
   ``"Hello"`` a string, ``"=A1*2"`` a formula.

.. method:: update_cell_with_text(sheet: int, row: int, column: int, value: str)

   Sets a string value without input parsing (``"123"`` stays a string).

.. method:: update_cell_with_number(sheet: int, row: int, column: int, value: float)
.. method:: update_cell_with_bool(sheet: int, row: int, column: int, value: bool)
.. method:: update_cell_with_formula(sheet: int, row: int, column: int, formula: str)
.. method:: set_user_array_formula(sheet, row, column, width, height, formula: str)

   Sets an array (spill) formula covering ``width`` x ``height`` cells.

.. method:: clear_cell_contents(sheet: int, row: int, column: int)

   Clears the content of a single cell, keeping the formatting.

.. method:: range_clear_contents(sheet, start_row, start_column, end_row, end_column)
.. method:: range_clear_all(sheet, start_row, start_column, end_row, end_column)

Getting values
^^^^^^^^^^^^^^

.. method:: get_cell_value(sheet: int, row: int, column: int)

   Returns the value of a cell as a native Python value: ``None``, ``str``,
   ``float`` or ``bool``.

.. method:: get_cell_value_by_ref(cell_ref: str)

   Same, with a reference like ``"Sheet1!C4"``.

.. method:: get_formatted_cell_value(sheet: int, row: int, column: int) -> str

   Returns the value formatted with the cell's number format (i.e. ``"$5.75"``).

.. method:: get_cell_content(sheet: int, row: int, column: int) -> str

   Returns the content as seen in the editor: the formula if there is one, the
   raw value otherwise.

.. method:: get_cell_formula(sheet: int, row: int, column: int) -> str | None
.. method:: get_cell_type(sheet: int, row: int, column: int) -> CellType
.. method:: is_empty_cell(sheet: int, row: int, column: int) -> bool
.. method:: get_all_cells() -> list[tuple[int, int, int]]

   Returns all non-empty cells as ``(sheet, row, column)`` tuples.

.. method:: get_sheet_dimensions(sheet: int) -> (int, int, int, int)

   Returns ``(min_row, max_row, min_column, max_column)`` of the non-empty
   cells; ``(1, 1, 1, 1)`` for an empty sheet.

.. method:: get_sheet_markup(sheet: int) -> str

   Returns a markdown-like table of the sheet (formulas, not values), useful
   for debugging and tests.

Styles
^^^^^^

Styles are dictionaries; see :doc:`objects`.

.. method:: get_cell_style(sheet: int, row: int, column: int) -> dict
.. method:: set_cell_style(sheet: int, row: int, column: int, style: dict)
.. method:: get_column_style(sheet: int, column: int) -> dict | None
.. method:: set_column_style(sheet: int, column: int, style: dict)
.. method:: delete_column_style(sheet: int, column: int)
.. method:: get_row_style(sheet: int, row: int) -> dict | None
.. method:: set_row_style(sheet: int, row: int, style: dict)
.. method:: delete_row_style(sheet: int, row: int)

Rows and columns
^^^^^^^^^^^^^^^^

.. method:: insert_rows(sheet: int, row: int, row_count: int)
.. method:: insert_columns(sheet: int, column: int, column_count: int)
.. method:: delete_rows(sheet: int, row: int, row_count: int)
.. method:: delete_columns(sheet: int, column: int, column_count: int)
.. method:: get_column_width(sheet: int, column: int) -> float
.. method:: get_row_height(sheet: int, row: int) -> float
.. method:: set_column_width(sheet: int, column: int, width: float)
.. method:: set_row_height(sheet: int, row: int, height: float)
.. method:: set_column_hidden(sheet: int, column: int, hidden: bool)
.. method:: set_row_hidden(sheet: int, row: int, hidden: bool)
.. method:: is_column_hidden(sheet: int, column: int) -> bool
.. method:: is_row_hidden(sheet: int, row: int) -> bool
.. method:: get_frozen_rows_count(sheet: int) -> int
.. method:: get_frozen_columns_count(sheet: int) -> int
.. method:: set_frozen_rows_count(sheet: int, row_count: int)
.. method:: set_frozen_columns_count(sheet: int, column_count: int)

Sheets
^^^^^^

.. method:: add_sheet(sheet_name: str)
.. method:: new_sheet()

   Adds a sheet with an automatically generated name.

.. method:: delete_sheet(sheet: int)
.. method:: rename_sheet(sheet: int, new_name: str)
.. method:: set_sheet_color(sheet: int, color)

   ``color`` is ``None``, ``"#RRGGBB"`` or ``[theme_index, tint]``.

.. method:: set_sheet_state(sheet: int, state: str)

   ``state`` is ``"visible"``, ``"hidden"`` or ``"veryHidden"``.

.. method:: get_worksheets_properties() -> list[dict]
.. method:: set_show_grid_lines(sheet: int, show_grid_lines: bool)

Defined names
^^^^^^^^^^^^^

.. method:: get_defined_name_list() -> list[dict]
.. method:: new_defined_name(name: str, scope: int | None, formula: str)
.. method:: update_defined_name(name, scope, new_name, new_scope, new_formula)
.. method:: delete_defined_name(name: str, scope: int | None)

Workbook properties
^^^^^^^^^^^^^^^^^^^

.. method:: get_theme() -> dict
.. method:: set_theme(theme: dict)
.. method:: get_timezone() -> str
.. method:: set_timezone(timezone: str)
.. method:: get_locale() -> str
.. method:: set_locale(locale: str)
.. method:: get_language() -> str
.. method:: set_language(language: str)
.. method:: get_fmt_settings() -> dict
