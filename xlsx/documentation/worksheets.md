Worksheets
==========

All the sheets in the workbook are in `xl/worksheets/sheet*.xlm` and represent the single most important files for us.

An example, ignoring for now the most important part `sheetData`

```xml
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:xdr="http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing" xmlns:x14="http://schemas.microsoft.com/office/spreadsheetml/2009/9/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:x14ac="http://schemas.microsoft.com/office/spreadsheetml/2009/9/ac" xmlns:xr="http://schemas.microsoft.com/office/spreadsheetml/2014/revision" xmlns:xr2="http://schemas.microsoft.com/office/spreadsheetml/2015/revision2" xmlns:xr3="http://schemas.microsoft.com/office/spreadsheetml/2016/revision3" mc:Ignorable="x14ac xr xr2 xr3" xr:uid="{65AA7E95-0880-433A-9B1F-8563DB0FF1B5}">
<dimension ref="A1:O33"/>
<sheetViews>
<sheetView workbookViewId="0">
<selection activeCell="I6" sqref="I6"/>
</sheetView>
</sheetViews>
<sheetFormatPr defaultRowHeight="14.5" x14ac:dyDescent="0.35"/>
<cols>
<col min="5" max="5" width="38.26953125" customWidth="1"/>
<col min="6" max="6" width="9.1796875" style="1"/>
<col min="8" max="8" width="4" customWidth="1"/>
</cols>
<sheetData>
...
</sheetData>
<mergeCells count="2">
<mergeCell ref="K7:L10"/>
<mergeCell ref="H18:J20"/>
</mergeCells>
<pageMargins left="0.7" right="0.7" top="0.75" bottom="0.75" header="0.3" footer="0.3"/>
<pageSetup orientation="portrait" r:id="rId1"/>
<legacyDrawing r:id="rId2"/>
</worksheet>
```

For this file we can read the columns information, the sheet data and merged cells.
For now everything else is ignored and lost in IronCalc.

The sheetData is organized by rows:

```xml
<sheetData>
<row r="1" spans="1:2" x14ac:dyDescent="0.35">
    <c r="A1" t="s">
        <v>0</v>
    </c>
    <c r="C1">
        <v>1</v>
    </c>
</row>
<row r="2" spans="1:2" x14ac:dyDescent="0.35">
    <c r="A2">
        <v>222</v>
    </c>
    <c r="C2">
        <v>2</v>
    </c>
</row>
</sheetData>
```

In IronCalc the `spans` (an Excel optimization) is not used. The `dyDescent` property is also ignore in `IronCalc`,