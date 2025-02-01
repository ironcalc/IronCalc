Objects
-------

The following examples


``WorkbookError`` 
^^^^^^^^^^^^^^^^^
Exceptions of type ``WorkbookError`` are raised whenever there is a problem with 
the workbook (e.g., invalid parameters, file I/O error, or even a Rust panic). 
You can catch these exceptions in Python as follows:

.. code-block:: python

   from ironcalc import WorkbookError

   try:
       # Some operation on PyModel
       pass
   except WorkbookError as e:
       print("Caught a workbook error:", e)

``PyCellType``
^^^^^^^^^^^^^^
Represents the type of a cell (e.g., number, string, boolean, etc.). You can 
check the type of a cell with :meth:`PyModel.get_cell_type`.

``PyStyle``
^^^^^^^^^^^
Represents the style of a cell (font, bold, number formats, alignment, etc.). 
You can get/set these styles with :meth:`PyModel.get_cell_style` 
and :meth:`PyModel.set_cell_style`.