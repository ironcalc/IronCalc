import { UserModel, Model } from './index.js'


function testUserModel() {
    const model = new UserModel("Workbook1", "en", "UTC");

    model.setUserInput(0, 1, 1, "=1+1");
    let t = model.getFormattedCellValue(0, 1, 1);

    console.log('From native', t);

    let t2 = model.getCellStyle(0, 1, 1);
    console.log('From native', t2);
}

function testModel() {
    const model = Model.fromXlsx("example.xlsx", "en", "UTC");
    const style = model.getCellStyle(0, 1, 6);
    console.log(style);

    const quantum = model.getFormattedCellValue(0, 14, 4);
    console.log(quantum);
    
}


testUserModel();
testModel();