
Usage Examples
--------------

All the examples on this page live in ``docs/examples`` in the repository and
are run as part of the test suite, so they always work with the current
version.

The two APIs
^^^^^^^^^^^^

IronCalc ships two APIs:

* The **user API** (:class:`UserModel`): the same high level API used by the
  IronCalc web application. Every action evaluates the workbook, keeps
  undo/redo history and produces diffs for collaboration.
* The **raw API** (:class:`Model`): a low level API. Nothing is evaluated
  automatically (call :meth:`Model.evaluate` yourself), there is no undo/redo
  and no diffs are produced.

.. literalinclude:: examples/simple.py
   :language: python

A styled report
^^^^^^^^^^^^^^^

Building a small sales report: bold headers, background colors, currency
formats, borders and column widths, saved as xlsx.

.. literalinclude:: examples/styled_report.py
   :language: python

Conditional formatting
^^^^^^^^^^^^^^^^^^^^^^

Color scales and cell-is rules. Rules are plain dictionaries with the same
shape used by the IronCalc web application.

.. literalinclude:: examples/conditional_formatting.py
   :language: python

Reading an xlsx file
^^^^^^^^^^^^^^^^^^^^

Values come back as native Python types (``None``, ``str``, ``float`` or
``bool``).

.. literalinclude:: examples/read_xlsx.py
   :language: python

Importing CSV data
^^^^^^^^^^^^^^^^^^

.. literalinclude:: examples/csv_import.py
   :language: python

Collaboration with diffs
^^^^^^^^^^^^^^^^^^^^^^^^

Two models can be kept in sync by exchanging binary diffs, the same mechanism
the IronCalc web application uses for real time collaboration.

.. literalinclude:: examples/collaboration.py
   :language: python
