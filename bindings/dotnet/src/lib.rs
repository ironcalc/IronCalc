use ironcalc::base::cell::CellValue;
use ironcalc::base::Model;
use std::ffi::c_char;

#[repr(C)]
pub struct ModelContext;

struct InternalModelContext {
    model: Model,
}

#[no_mangle]
pub extern "C" fn from_bytes(
    buffer: *const u8,
    length: i32,
    locale: *const c_char,
    timezone: *const c_char,
    name: *const c_char,
) -> *mut ModelContext {
    let model = unsafe {
        let slice = std::slice::from_raw_parts(buffer, length as usize);
        let locale_str = std::ffi::CStr::from_ptr(locale).to_string_lossy();
        let timezone_str = std::ffi::CStr::from_ptr(timezone).to_string_lossy();

        let name_str = match name.as_ref() {
            Some(name) => {
                &std::ffi::CStr::from_ptr(name).to_string_lossy()
            },
            _ => "Book1",
        };

        let workbook = ironcalc::import::load_from_xlsx_bytes(slice, &name_str, &locale_str, &timezone_str)
            .expect("Couldn't load from xlsx");
        Model::from_workbook(workbook).expect("couldn't create model from xlsx")
    };

    let ctx = Box::new(InternalModelContext { model });

    Box::into_raw(ctx) as *mut ModelContext
}

#[no_mangle]
pub extern "C" fn new_empty(locale: *const c_char, timezone: *const c_char) -> *mut ModelContext {
    let model = unsafe {
        let locale_str = std::ffi::CStr::from_ptr(locale).to_string_lossy();
        let timezone_str = std::ffi::CStr::from_ptr(timezone).to_string_lossy();
        Model::new_empty("", &locale_str, &timezone_str).expect("couldn't create model")
    };

    let ctx = Box::new(InternalModelContext { model });

    Box::into_raw(ctx) as *mut ModelContext
}

#[no_mangle]
pub unsafe extern "C" fn evaluate(context: *mut ModelContext) {
    let mut ctx = Box::from_raw(context as *mut InternalModelContext);
    ctx.model.evaluate();
    Box::into_raw(ctx) as *mut ModelContext;
}

#[no_mangle]
pub unsafe extern "C" fn get_value(
    context: *mut ModelContext,
    sheet: i32,
    row: i32,
    col: i32,
) -> i32 {
    let ctx = Box::from_raw(context as *mut InternalModelContext);

    let value = ctx
        .model
        .get_cell_value_by_index(sheet as u32, row, col)
        .expect("couldn't get sheet");
    let return_value = match value {
        CellValue::Number(x) => x as i32,
        _ => 0,
    };

    Box::into_raw(ctx) as *mut ModelContext;

    return_value
}

#[no_mangle]
pub unsafe extern "C" fn set_value(
    context: *mut ModelContext,
    sheet: i32,
    row: i32,
    col: i32,
    value: i32,
) {
    let mut ctx = Box::from_raw(context as *mut InternalModelContext);

    ctx.model
        .set_user_input(sheet as u32, row, col, format!("{value}"))
        .expect("couldn't set user input");

    Box::into_raw(ctx) as *mut ModelContext;
}

#[no_mangle]
pub unsafe extern "C" fn dispose(context: *mut ModelContext) {
    _ = Box::from_raw(context as *mut InternalModelContext);
}
