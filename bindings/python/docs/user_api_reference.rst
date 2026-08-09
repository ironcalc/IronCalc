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

.. method:: UserModel.save_to_xlsx(file: str)

   Saves the workbook to an xlsx file.

.. method:: UserModel.save_to_icalc(file: str)

   Saves the workbook to a file in the internal binary ic format.

.. method:: UserModel.to_bytes() -> bytes

   Returns the workbook as bytes in the internal binary ic format.

Collaboration
^^^^^^^^^^^^^

.. method:: UserModel.flush_send_queue() -> bytes

   Returns (and clears) the queue of binary diffs produced by local edits.

.. method:: UserModel.apply_external_diffs(external_diffs: bytes)

   Applies a list of diffs produced by another model's ``flush_send_queue``.

Undo, redo and evaluation
^^^^^^^^^^^^^^^^^^^^^^^^^

.. method:: UserModel.undo()
.. method:: UserModel.redo()
.. method:: UserModel.can_undo() -> bool
.. method:: UserModel.can_redo() -> bool

.. method:: UserModel.pause_evaluation()

   Pauses the automatic evaluation performed after each change; useful when
   entering large amounts of data.

.. method:: UserModel.resume_evaluation()
.. method:: UserModel.evaluate()

   Forces an evaluation (only needed while the evaluation is paused).

Cell values
^^^^^^^^^^^

.. method:: UserModel.set_user_input(sheet: int, row: int, column: int, value: str)

   Sets the input of a cell as a user would type it: ``"3.5"`` is a number,
   ``"Hello"`` a string, ``"=A1*2"`` a formula.

.. method:: UserModel.set_user_array_formula(sheet: int, row: int, column: int, width: int, height: int, formula: str)

   Sets an array (spill) formula covering ``width`` x ``height`` cells.

.. method:: UserModel.get_cell_content(sheet: int, row: int, column: int) -> str

   Returns the content of a cell as seen in the editor: the formula if there
   is one, the raw value otherwise.

.. method:: UserModel.get_formatted_cell_value(sheet: int, row: int, column: int) -> str

   Returns the value formatted with the cell's number format (i.e. ``"$5.75"``).

.. method:: UserModel.get_cell_type(sheet: int, row: int, column: int) -> CellType
.. method:: UserModel.get_cell_array_structure(sheet: int, row: int, column: int)

   Returns information about the array (spill) structure of a cell.

.. method:: UserModel.get_sheet_dimensions(sheet: int) -> (int, int, int, int)

   Returns ``(min_row, max_row, min_column, max_column)`` of the non-empty
   cells; ``(1, 1, 1, 1)`` for an empty sheet.

Ranges
^^^^^^

.. method:: UserModel.range_clear_all(sheet, start_row, start_column, end_row, end_column)

   Clears contents and formatting.

.. method:: UserModel.range_clear_contents(sheet, start_row, start_column, end_row, end_column)

   Clears contents, keeps formatting.

.. method:: UserModel.range_clear_formatting(sheet, start_row, start_column, end_row, end_column)

   Clears formatting, keeps contents.

.. method:: UserModel.auto_fill_rows(sheet, start_row, start_column, end_row, end_column, to_row: int)

   Extends the content of the source area down (or up) until ``to_row``, like
   dragging the fill handle.

.. method:: UserModel.auto_fill_columns(sheet, start_row, start_column, end_row, end_column, to_column: int)

Styles
^^^^^^

.. method:: UserModel.update_range_style(sheet, start_row, start_column, end_row, end_column, style_path: str, value: str)

   Updates one style property in every cell of the range. ``style_path``
   examples: ``"font.b"``, ``"font.color"``, ``"fill.color"``,
   ``"alignment.horizontal"``, ``"num_fmt"``. The value is always a string:
   ``"true"``, ``"#FF5566"``, ``"center"``, ``"#,##0.00"``.

.. method:: UserModel.get_cell_style(sheet: int, row: int, column: int) -> dict

   Returns the style of the cell as a dictionary.

.. method:: UserModel.get_extended_cell_style(sheet: int, row: int, column: int) -> dict

   Returns the style with any conditional formatting overlay applied, plus
   icon-set / data-bar decorations.

.. method:: UserModel.set_area_with_border(sheet, start_row, start_column, end_row, end_column, border_area: dict)

   Applies a border to an area. ``border_area`` is
   ``{"item": {"style": "thin", "color": "#000000"}, "type": "All"}`` where
   ``type`` is one of ``All``, ``Inner``, ``Outer``, ``Top``, ``Right``,
   ``Bottom``, ``Left``, ``CenterH``, ``CenterV``, ``None``.

.. method:: UserModel.on_paste_styles(styles: list[list[dict]])

   Pastes a matrix of styles starting at the selected cell.

Named styles
^^^^^^^^^^^^

.. method:: UserModel.get_named_style_list() -> list[str]
.. method:: UserModel.get_named_style(name: str) -> dict
.. method:: UserModel.get_named_style_includes(name: str) -> dict
.. method:: UserModel.create_named_style(name: str, style: dict, includes: dict | None = None)

   Creates a named style. ``includes`` selects which formatting categories the
   style carries (Excel's "Style Includes" checkboxes); ``None`` means all.

.. method:: UserModel.update_named_style(name: str, new_name: str, style: dict, includes: dict | None = None)
.. method:: UserModel.delete_named_style(name: str)
.. method:: UserModel.get_builtin_named_styles() -> list[dict]

   Returns all Excel built-in named styles as ``{"name", "style"}`` entries.

.. method:: UserModel.on_apply_named_style(name: str)

   Applies a named style to the current selection, adding it from the
   built-ins if needed.

Conditional formatting
^^^^^^^^^^^^^^^^^^^^^^

Rules are dictionaries; see :doc:`objects` for the shapes.

.. method:: UserModel.get_conditional_formatting_list(sheet: int) -> list[dict]

   Returns entries with ``range``, ``cf_rule``, ``priority`` and ``index``,
   sorted by priority descending.

.. method:: UserModel.add_conditional_formatting(sheet: int, range: str, rule: dict)

   Adds a rule to a range given as a string like ``"A1:B10"``.

.. method:: UserModel.update_conditional_formatting(sheet: int, index: int, new_range: str, new_rule: dict)
.. method:: UserModel.delete_conditional_formatting(sheet: int, index: int)
.. method:: UserModel.get_dxf_for_conditional_formatting(sheet: int, index: int) -> dict | None

   Returns the differential style the rule applies, if it has one.

.. method:: UserModel.raise_conditional_formatting_priority(sheet: int, index: int)
.. method:: UserModel.lower_conditional_formatting_priority(sheet: int, index: int)

Sheets
^^^^^^

.. method:: UserModel.new_sheet()

   Adds a sheet with an automatically generated name.

.. method:: UserModel.delete_sheet(sheet: int)
.. method:: UserModel.duplicate_sheet(sheet: int)
.. method:: UserModel.rename_sheet(sheet: int, name: str)
.. method:: UserModel.move_sheet(sheet: int, new_index: int)
.. method:: UserModel.hide_sheet(sheet: int)
.. method:: UserModel.unhide_sheet(sheet: int)
.. method:: UserModel.set_sheet_color(sheet: int, color)

   ``color`` is ``None``, ``"#RRGGBB"`` or ``[theme_index, tint]``.

.. method:: UserModel.get_worksheets_properties() -> list[dict]

   One entry per sheet with ``name``, ``state``, ``sheet_id`` and ``color``.

.. method:: UserModel.set_show_grid_lines(sheet: int, show_grid_lines: bool)
.. method:: UserModel.get_show_grid_lines(sheet: int) -> bool

Rows and columns
^^^^^^^^^^^^^^^^

.. method:: UserModel.insert_rows(sheet: int, row: int, row_count: int)
.. method:: UserModel.insert_columns(sheet: int, column: int, column_count: int)
.. method:: UserModel.delete_rows(sheet: int, row: int, row_count: int)
.. method:: UserModel.delete_columns(sheet: int, column: int, column_count: int)
.. method:: UserModel.move_rows(sheet: int, row: int, row_count: int, delta: int)
.. method:: UserModel.move_columns(sheet: int, column: int, column_count: int, delta: int)
.. method:: UserModel.get_row_height(sheet: int, row: int) -> float
.. method:: UserModel.get_column_width(sheet: int, column: int) -> float
.. method:: UserModel.set_rows_height(sheet: int, row_start: int, row_end: int, height: float)
.. method:: UserModel.set_columns_width(sheet: int, column_start: int, column_end: int, width: float)
.. method:: UserModel.set_rows_hidden(sheet: int, row_start: int, row_end: int, hidden: bool)
.. method:: UserModel.set_columns_hidden(sheet: int, column_start: int, column_end: int, hidden: bool)
.. method:: UserModel.get_frozen_rows_count(sheet: int) -> int
.. method:: UserModel.get_frozen_columns_count(sheet: int) -> int
.. method:: UserModel.set_frozen_rows_count(sheet: int, count: int)
.. method:: UserModel.set_frozen_columns_count(sheet: int, count: int)
.. method:: UserModel.get_last_non_empty_in_row_before_column(sheet, row, column) -> int | None
.. method:: UserModel.get_first_non_empty_in_row_after_column(sheet, row, column) -> int | None

Defined names
^^^^^^^^^^^^^

.. method:: UserModel.get_defined_name_list() -> list[dict]

   Entries have ``name``, ``scope`` (sheet index or ``None`` for global) and
   ``formula``.

.. method:: UserModel.new_defined_name(name: str, scope: int | None, formula: str)
.. method:: UserModel.update_defined_name(name, scope, new_name, new_scope, new_formula)
.. method:: UserModel.delete_defined_name(name: str, scope: int | None)
.. method:: UserModel.is_valid_defined_name(name: str, scope: int | None, formula: str)

   Raises :class:`WorkbookError` if the name or formula is not valid.

Selection
^^^^^^^^^

Some operations (applying named styles, pasting styles, copying to the
clipboard) act on the current selection.

.. method:: UserModel.get_selected_sheet() -> int
.. method:: UserModel.get_selected_cell() -> (int, int, int)
.. method:: UserModel.get_selected_view() -> dict
.. method:: UserModel.set_selected_sheet(sheet: int)
.. method:: UserModel.set_selected_cell(row: int, column: int)
.. method:: UserModel.set_selected_range(start_row, start_column, end_row, end_column)

   The selected cell must be one of the corners of the range; call
   :meth:`UserModel.set_selected_cell` first.

Clipboard
^^^^^^^^^

.. method:: UserModel.copy_to_clipboard() -> dict

   Copies the selected area. The result has ``csv`` (tab separated text),
   ``data`` (internal representation), ``sheet`` and ``range``.

.. method:: UserModel.paste_from_clipboard(source_sheet: int, source_range: tuple, clipboard: dict, is_cut: bool)

   Pastes data copied with ``copy_to_clipboard`` into the selected area,
   adjusting relative references. Pass ``clipboard["data"]`` as the
   ``clipboard`` argument.

.. method:: UserModel.paste_csv_string(sheet, start_row, start_column, end_row, end_column, csv: str)

   Pastes tab separated text starting at the top-left corner of the area.

Workbook properties
^^^^^^^^^^^^^^^^^^^

.. method:: UserModel.get_name() -> str
.. method:: UserModel.set_name(name: str)
.. method:: UserModel.get_timezone() -> str
.. method:: UserModel.set_timezone(timezone: str)
.. method:: UserModel.get_locale() -> str
.. method:: UserModel.set_locale(locale: str)
.. method:: UserModel.get_language() -> str
.. method:: UserModel.set_language(language: str)
.. method:: UserModel.get_fmt_settings() -> dict

   Returns locale dependent formatting settings (currency, date formats, ...).

.. method:: UserModel.get_theme() -> dict
.. method:: UserModel.set_theme(theme: dict)
.. method:: UserModel.resolve_color(color) -> str

   Resolves a color to a CSS hex string using the current workbook theme;
   returns ``""`` for no color.
