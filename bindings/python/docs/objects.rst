Objects
-------

``WorkbookError``
^^^^^^^^^^^^^^^^^
Exceptions of type ``WorkbookError`` are raised whenever there is a problem
with the workbook (e.g., invalid parameters, file I/O error, or even a Rust
panic). You can catch these exceptions in Python as follows:

.. code-block:: python

   from ironcalc import WorkbookError

   try:
       model.delete_sheet(99)
   except WorkbookError as e:
       print("Caught a workbook error:", e)

``CellType``
^^^^^^^^^^^^
An enum with the type of a cell content, following Excel's ``TYPE()``
convention: ``Number``, ``Text``, ``LogicalValue``, ``ErrorValue``, ``Array``
and ``CompoundData``. Returned by ``get_cell_type``.

Style dictionaries
^^^^^^^^^^^^^^^^^^
Complex values (styles, conditional formatting rules, themes, borders, ...)
cross the Rust/Python boundary as plain dictionaries and lists, with exactly
the same shapes used by the IronCalc wasm bindings and web application.

A cell style looks like this:

.. code-block:: python

   {
       "num_fmt": "$#,##0.00",
       "font": {"b": True, "sz": 12, "name": "Inter", "color": "#333333"},
       "fill": {"color": "#FFFF00"},
       "border": {"top": {"style": "thin", "color": "#000000"}},
       "alignment": {"horizontal": "center", "vertical": "top", "wrap_text": False},
       "quote_prefix": False,
   }

Note that boolean flags that are false (like ``font.b``) and empty colors are
omitted from the dictionaries, so prefer ``style["font"].get("b")`` over
``style["font"]["b"]`` when reading.

The recommended way to modify styles with the user API is
:meth:`UserModel.update_range_style`, which takes a dotted path and a string
value:

.. code-block:: python

   model.update_range_style(0, 1, 1, 5, 5, "font.b", "true")
   model.update_range_style(0, 1, 1, 5, 5, "fill.color", "#FFDD00")
   model.update_range_style(0, 1, 1, 5, 5, "alignment.horizontal", "center")
   model.update_range_style(0, 1, 1, 5, 5, "num_fmt", "#,##0.00")

With the raw API you read a style dictionary, modify it and write it back:

.. code-block:: python

   style = model.get_cell_style(0, 1, 1)
   style["font"]["b"] = True
   model.set_cell_style(0, 1, 1, style)

Colors
^^^^^^
A color is either ``None`` (no color), a ``"#RRGGBB"`` string, or a
``[theme_index, tint]`` pair referencing the workbook theme.

Conditional formatting rules
^^^^^^^^^^^^^^^^^^^^^^^^^^^^
Rules are dictionaries with a ``type`` key. Some examples:

.. code-block:: python

   # Color scale from red to green
   {
       "type": "ColorScale",
       "thresholds": [
           {"cfvo": "Min", "color": "#F8696B"},
           {"cfvo": {"Percentile": 50}, "color": "#FFEB84"},
           {"cfvo": "Max", "color": "#63BE7B"},
       ],
   }

   # Highlight cells greater than 90
   {
       "type": "CellIs",
       "operator": "GreaterThan",  # Equal, LessThan, Between, ...
       "formula": "90",
       "formula2": None,           # used by Between / NotBetween
       "format": {"fill": {"color": "#FFC7CE"}, "font": {"b": True}},
       "stop_if_true": False,
   }

The ``format`` value is a *differential* style (dxf): only the keys present
are overlaid on the cell style when the rule matches.
