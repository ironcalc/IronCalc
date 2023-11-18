Documentation
=============

An `xlsx` is a zip file containing a set of folders and `xml` files. The IronCalc json structure mimics the relevant parts of the Excel zip.
Although the xlsx structure is quite complicated, it's essentials regarding the spreadsheet technology are easier to grasp.

The simplest workbook folder structure might look like this:

```
docProps
    app.xml
    core.xml

_rels
    .rels

xl
    _rels
        workbook.xml.rels
    theme
        theme1.xml
    worksheets
        sheet1.xml
    calcChain.xml
    styles.xml
    workbook.xml
    sharedStrings.xml

[Content_Types].xml
```

Note that more complicated workbooks will have many more files and folders.
For instance charts, pivot tables, comments, tables,...

The relevant json structure in IronCalc will be:


```json
{
    "name": "Workbook1",
    "defined_names": [],
    "shared_strings": [],
    "worksheets": [],
    "styles": {
        "num_fmts": [],
        "fonts": [],
        "fills": [],
        "borders": [],
        "cell_style_xfs": [],
        "cell_styles" : [],
        "cell_xfs": []
    }
}
```

Note that there is not a 1-1 correspondence but there is a close resemblance.



SpreadsheetML
-------------
International standard (Four edition 2016-11-01): ECMA-376, ISO/IEC 29500-1
* [iso](https://standards.iso.org/ittf/PubliclyAvailableStandards/c071691_ISO_IEC_29500-1_2016.zip)
* [ecma](http://www.ecma-international.org/publications/standards/Ecma-376.htm)