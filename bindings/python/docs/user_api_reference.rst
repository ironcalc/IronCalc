User API Reference
------------------

The user API (:class:`UserModel`) is the high level API used by the IronCalc
web application. Every action evaluates the workbook, keeps undo/redo history
and produces diffs for collaboration.

Conventions:

* ``sheet`` indexes are 0-based; ``row`` and ``column`` indexes are 1-based
  (column "A" is 1).
* Range arguments are always ``(sheet, start_row, start_column, end_row,
  end_column)``, all bounds inclusive.
* Complex values (styles, rules, themes) are plain dictionaries; see
  :doc:`objects`.
* All methods raise :class:`WorkbookError` on invalid input.

Constructors
^^^^^^^^^^^^

.. class:: UserModel(name, locale="en", tz="UTC", language_id="en")

   Creates an empty workbook.

.. staticmethod:: UserModel.from_xlsx(file_path, locale="en", tz="UTC", language_id="en") -> UserModel
.. staticmethod:: UserModel.from_icalc(file_name, language_id="en") -> UserModel
.. staticmethod:: UserModel.from_bytes(bytes, language_id="en") -> UserModel

Persistence
^^^^^^^^^^^

.. method:: save_to_xlsx(file: str)

   Saves the workbook to an xlsx file.

.. method:: save_to_icalc(file: str)

   Saves the workbook to a file in the internal binary ic format.

.. method:: to_bytes() -> bytes

   Returns the workbook as bytes in the internal binary ic format.

Collaboration
^^^^^^^^^^^^^

.. method:: flush_send_queue() -> bytes

   Returns (and clears) the queue of binary diffs produced by local edits.

.. method:: apply_external_diffs(external_diffs: bytes)

   Applies a list of diffs produced by another model's ``flush_send_queue``.

Undo, redo and evaluation
^^^^^^^^^^^^^^^^^^^^^^^^^

.. method:: undo()
.. method:: redo()
.. method:: can_undo() -> bool
.. method:: can_redo() -> bool

.. method:: pause_evaluation()

   Pauses the automatic evaluation performed after each change; useful when
   entering large amounts of data.

.. method:: resume_evaluation()
.. method:: evaluate()

   Forces an evaluation (only needed while the evaluation is paused).

Cell values
^^^^^^^^^^^

.. method:: set_user_input(sheet: int, row: int, column: int, value: str)

   Sets the input of a cell as a user would type it: ``"3.5"`` is a number,
   ``"Hello"`` a string, ``"=A1*2"`` a formula.

.. method:: set_user_array_formula(sheet: int, row: int, column: int, width: int, height: int, formula: str)

   Sets an array (spill) formula covering ``width`` x ``height`` cells.

.. method:: get_cell_content(sheet: int, row: int, column: int) -> str

   Returns the content of a cell as seen in the editor: the formula if there
   is one, the raw value otherwise.

.. method:: get_formatted_cell_value(sheet: int, row: int, column: int) -> str

   Returns the value formatted with the cell's number format (i.e. ``"$5.75"``).

.. method:: get_cell_type(sheet: int, row: int, column: int) -> CellType
.. method:: get_cell_array_structure(sheet: int, row: int, column: int)

   Returns information about the array (spill) structure of a cell.

.. method:: get_sheet_dimensions(sheet: int) -> (int, int, int, int)

   Returns ``(min_row, max_row, min_column, max_column)`` of the non-empty
   cells; ``(1, 1, 1, 1)`` for an empty sheet.

Ranges
^^^^^^

.. method:: range_clear_all(sheet, start_row, start_column, end_row, end_column)

   Clears contents and formatting.

.. method:: range_clear_contents(sheet, start_row, start_column, end_row, end_column)

   Clears contents, keeps formatting.

.. method:: range_clear_formatting(sheet, start_row, start_column, end_row, end_column)

   Clears formatting, keeps contents.

.. method:: auto_fill_rows(sheet, start_row, start_column, end_row, end_column, to_row: int)

   Extends the content of the source area down (or up) until ``to_row``, like
   dragging the fill handle.

.. method:: auto_fill_columns(sheet, start_row, start_column, end_row, end_column, to_column: int)

Styles
^^^^^^

.. method:: update_range_style(sheet, start_row, start_column, end_row, end_column, style_path: str, value: str)

   Updates one style property in every cell of the range. ``style_path``
   examples: ``"font.b"``, ``"font.color"``, ``"fill.color"``,
   ``"alignment.horizontal"``, ``"num_fmt"``. The value is always a string:
   ``"true"``, ``"#FF5566"``, ``"center"``, ``"#,##0.00"``.

.. method:: get_cell_style(sheet: int, row: int, column: int) -> dict

   Returns the style of the cell as a dictionary.

.. method:: get_extended_cell_style(sheet: int, row: int, column: int) -> dict

   Returns the style with any conditional formatting overlay applied, plus
   icon-set / data-bar decorations.

.. method:: set_area_with_border(sheet, start_row, start_column, end_row, end_column, border_area: dict)

   Applies a border to an area. ``border_area`` is
   ``{"item": {"style": "thin", "color": "#000000"}, "type": "All"}`` where
   ``type`` is one of ``All``, ``Inner``, ``Outer``, ``Top``, ``Right``,
   ``Bottom``, ``Left``, ``CenterH``, ``CenterV``, ``None``.

.. method:: on_paste_styles(styles: list[list[dict]])

   Pastes a matrix of styles starting at the selected cell.

Named styles
^^^^^^^^^^^^

.. method:: get_named_style_list() -> list[str]
.. method:: get_named_style(name: str) -> dict
.. method:: get_named_style_includes(name: str) -> dict
.. method:: create_named_style(name: str, style: dict, includes: dict | None = None)

   Creates a named style. ``includes`` selects which formatting categories the
   style carries (Excel's "Style Includes" checkboxes); ``None`` means all.

.. method:: update_named_style(name: str, new_name: str, style: dict, includes: dict | None = None)
.. method:: delete_named_style(name: str)
.. method:: get_builtin_named_styles() -> list[dict]

   Returns all Excel built-in named styles as ``{"name", "style"}`` entries.

.. method:: on_apply_named_style(name: str)

   Applies a named style to the current selection, adding it from the
   built-ins if needed.

Conditional formatting
^^^^^^^^^^^^^^^^^^^^^^

Rules are dictionaries; see :doc:`objects` for the shapes.

.. method:: get_conditional_formatting_list(sheet: int) -> list[dict]

   Returns entries with ``range``, ``cf_rule``, ``priority`` and ``index``,
   sorted by priority descending.

.. method:: add_conditional_formatting(sheet: int, range: str, rule: dict)

   Adds a rule to a range given as a string like ``"A1:B10"``.

.. method:: update_conditional_formatting(sheet: int, index: int, new_range: str, new_rule: dict)
.. method:: delete_conditional_formatting(sheet: int, index: int)
.. method:: get_dxf_for_conditional_formatting(sheet: int, index: int) -> dict | None

   Returns the differential style the rule applies, if it has one.

.. method:: raise_conditional_formatting_priority(sheet: int, index: int)
.. method:: lower_conditional_formatting_priority(sheet: int, index: int)

Sheets
^^^^^^

.. method:: new_sheet()

   Adds a sheet with an automatically generated name.

.. method:: delete_sheet(sheet: int)
.. method:: duplicate_sheet(sheet: int)
.. method:: rename_sheet(sheet: int, name: str)
.. method:: move_sheet(sheet: int, new_index: int)
.. method:: hide_sheet(sheet: int)
.. method:: unhide_sheet(sheet: int)
.. method:: set_sheet_color(sheet: int, color)

   ``color`` is ``None``, ``"#RRGGBB"`` or ``[theme_index, tint]``.

.. method:: get_worksheets_properties() -> list[dict]

   One entry per sheet with ``name``, ``state``, ``sheet_id`` and ``color``.

.. method:: set_show_grid_lines(sheet: int, show_grid_lines: bool)
.. method:: get_show_grid_lines(sheet: int) -> bool

Rows and columns
^^^^^^^^^^^^^^^^

.. method:: insert_rows(sheet: int, row: int, row_count: int)
.. method:: insert_columns(sheet: int, column: int, column_count: int)
.. method:: delete_rows(sheet: int, row: int, row_count: int)
.. method:: delete_columns(sheet: int, column: int, column_count: int)
.. method:: move_rows(sheet: int, row: int, row_count: int, delta: int)
.. method:: move_columns(sheet: int, column: int, column_count: int, delta: int)
.. method:: get_row_height(sheet: int, row: int) -> float
.. method:: get_column_width(sheet: int, column: int) -> float
.. method:: set_rows_height(sheet: int, row_start: int, row_end: int, height: float)
.. method:: set_columns_width(sheet: int, column_start: int, column_end: int, width: float)
.. method:: set_rows_hidden(sheet: int, row_start: int, row_end: int, hidden: bool)
.. method:: set_columns_hidden(sheet: int, column_start: int, column_end: int, hidden: bool)
.. method:: get_frozen_rows_count(sheet: int) -> int
.. method:: get_frozen_columns_count(sheet: int) -> int
.. method:: set_frozen_rows_count(sheet: int, count: int)
.. method:: set_frozen_columns_count(sheet: int, count: int)
.. method:: get_last_non_empty_in_row_before_column(sheet, row, column) -> int | None
.. method:: get_first_non_empty_in_row_after_column(sheet, row, column) -> int | None

Defined names
^^^^^^^^^^^^^

.. method:: get_defined_name_list() -> list[dict]

   Entries have ``name``, ``scope`` (sheet index or ``None`` for global) and
   ``formula``.

.. method:: new_defined_name(name: str, scope: int | None, formula: str)
.. method:: update_defined_name(name, scope, new_name, new_scope, new_formula)
.. method:: delete_defined_name(name: str, scope: int | None)
.. method:: is_valid_defined_name(name: str, scope: int | None, formula: str)

   Raises :class:`WorkbookError` if the name or formula is not valid.

Selection
^^^^^^^^^

Some operations (applying named styles, pasting styles, copying to the
clipboard) act on the current selection.

.. method:: get_selected_sheet() -> int
.. method:: get_selected_cell() -> (int, int, int)
.. method:: get_selected_view() -> dict
.. method:: set_selected_sheet(sheet: int)
.. method:: set_selected_cell(row: int, column: int)
.. method:: set_selected_range(start_row, start_column, end_row, end_column)

   The selected cell must be one of the corners of the range; call
   :meth:`set_selected_cell` first.

Clipboard
^^^^^^^^^

.. method:: copy_to_clipboard() -> dict

   Copies the selected area. The result has ``csv`` (tab separated text),
   ``data`` (internal representation), ``sheet`` and ``range``.

.. method:: paste_from_clipboard(source_sheet: int, source_range: tuple, clipboard: dict, is_cut: bool)

   Pastes data copied with ``copy_to_clipboard`` into the selected area,
   adjusting relative references. Pass ``clipboard["data"]`` as the
   ``clipboard`` argument.

.. method:: paste_csv_string(sheet, start_row, start_column, end_row, end_column, csv: str)

   Pastes tab separated text starting at the top-left corner of the area.

Workbook properties
^^^^^^^^^^^^^^^^^^^

.. method:: get_name() -> str
.. method:: set_name(name: str)
.. method:: get_timezone() -> str
.. method:: set_timezone(timezone: str)
.. method:: get_locale() -> str
.. method:: set_locale(locale: str)
.. method:: get_language() -> str
.. method:: set_language(language: str)
.. method:: get_fmt_settings() -> dict

   Returns locale dependent formatting settings (currency, date formats, ...).

.. method:: get_theme() -> dict
.. method:: set_theme(theme: dict)
.. method:: resolve_color(color) -> str

   Resolves a color to a CSS hex string using the current workbook theme;
   returns ``""`` for no color.
