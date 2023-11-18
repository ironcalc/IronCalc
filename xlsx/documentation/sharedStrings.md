Shared Strings
==============

In Excel the type of a cell that contains a string can be of one of three cases:
(see section 18.18.11 ST_CellType (Cell Type))

* 's' (A shared string)
* 'str' (A formula string)
* 'inlineStr' (An inline string)

This file holds a list of the shared strings. The following example contains two strings:

* Cell A1
* Cell A2

The second contains some internal formatting that in IronCalc is lost.

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="https://schemas.openxmlformats.org/spreadsheetml/2006/main" count="6" uniqueCount="2">
        
<si>
    <t>Cell A1</t>
</si>
<si>
    <r>
        <rPr>
            <sz val="11"/>
            <color rgb="FFFF0000"/>
            <rFont val="Calibri"/>
            <family val="2"/>
            <scheme val="minor"/>
        </rPr>
        <t>Cell</t>
    </r>
    <r>
        <rPr>
            <sz val="11"/>
            <color theme="1"/>
            <rFont val="Calibri"/>
            <family val="2"/>
            <scheme val="minor"/>
        </rPr>
        <t xml:space="preserve"> </t>
    </r>
    <r>
        <rPr>
            <b/>
            <sz val="11"/>
            <color theme="1"/>
            <rFont val="Calibri"/>
            <family val="2"/>
            <scheme val="minor"/>
        </rPr>
        <t>A2</t>
    </r>
</si>
</sst>
```

This will result in IronCalc in `shared_strings: ["Cell A1", "Cell A2"]`.

Note that the formatting we are loosing is different formatting within a cell. We can still format and style the full contents of a cell.

In this example there are two strings (`uniqueCount=2`) in the list but those strings are present in 6 cell across the workbook (`count=6`). Those parameters are not kept in IronCalc.

Another issue (a corner case) we will have in IronCalc is that we might end have repeated shared string in the list if the original Excel file has the same content is two cells with different formatting. That will mean that we end up using more memory than we need to but will not result in an error.