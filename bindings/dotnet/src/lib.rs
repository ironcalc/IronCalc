use ironcalc::base::cell::CellValue;
use ironcalc::base::Model;
use std::ffi::{c_char, CString};

#[repr(C)]
pub struct ModelContext;

#[repr(C)]
pub enum ModelContextErrorTag {
    XlsxError = 1,
    WorkbookError = 2,
    SetUserInputError = 3,
}

#[repr(C)]
pub struct ModelContextError {
    tag: ModelContextErrorTag,
    has_message: bool,
    message: *const c_char,
}

#[repr(C)]
pub struct CreateModelContextResult {
    model: *mut ModelContext,
    error: *mut ModelContextError,
    is_ok: bool,
}

impl CreateModelContextResult {
    fn create_error(message: String, tag: ModelContextErrorTag) -> CreateModelContextResult {
        let message_ptr = CString::new(message)
            .expect("Couldn't create CString")
            .into_raw();

        let error = Box::into_raw(Box::new(ModelContextError {
            tag,
            has_message: true,
            message: message_ptr,
        }));

        CreateModelContextResult {
            model: std::ptr::null_mut(),
            error,
            is_ok: false,
        }
    }
}

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
) -> CreateModelContextResult {
    let model = unsafe {
        let slice = std::slice::from_raw_parts(buffer, length as usize);
        let locale_str = std::ffi::CStr::from_ptr(locale).to_string_lossy();
        let timezone_str = std::ffi::CStr::from_ptr(timezone).to_string_lossy();

        let name_str = match name.as_ref() {
            Some(name) => &std::ffi::CStr::from_ptr(name).to_string_lossy(),
            _ => "",
        };

        let workbook = match ironcalc::import::load_from_xlsx_bytes(
            slice,
            &name_str,
            &locale_str,
            &timezone_str,
        ) {
            Ok(workbook) => workbook,
            Err(e) => {
                return CreateModelContextResult::create_error(
                    e.user_message(),
                    ModelContextErrorTag::XlsxError,
                );
            }
        };
        match Model::from_workbook(workbook) {
            Ok(model) => model,
            Err(error_msg) => {
                return CreateModelContextResult::create_error(
                    error_msg,
                    ModelContextErrorTag::WorkbookError,
                );
            }
        }
    };

    let ctx = Box::new(InternalModelContext { model });
    let ctx = Box::into_raw(ctx) as *mut ModelContext;

    CreateModelContextResult {
        model: ctx,
        error: std::ptr::null_mut(),
        is_ok: true,
    }
}

#[no_mangle]
pub extern "C" fn new_empty(
    name: *const c_char,
    locale: *const c_char,
    timezone: *const c_char,
) -> CreateModelContextResult {
    let model = unsafe {
        let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy();
        let locale_str = std::ffi::CStr::from_ptr(locale).to_string_lossy();
        let timezone_str = std::ffi::CStr::from_ptr(timezone).to_string_lossy();
        match Model::new_empty(&name_str, &locale_str, &timezone_str) {
            Ok(model) => model,
            Err(error_msg) => {
                return CreateModelContextResult::create_error(
                    error_msg,
                    ModelContextErrorTag::WorkbookError,
                );
            }
        }
    };

    let ctx = Box::new(InternalModelContext { model });
    let ctx = Box::into_raw(ctx) as *mut ModelContext;

    CreateModelContextResult {
        model: ctx,
        error: std::ptr::null_mut(),
        is_ok: true,
    }
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
pub unsafe extern "C" fn set_user_input(
    context: *mut ModelContext,
    sheet: u32,
    row: i32,
    col: i32,
    value: *const c_char,
) -> *mut ModelContextError {
    let mut ctx = Box::from_raw(context as *mut InternalModelContext);
    let value = std::ffi::CStr::from_ptr(value).to_string_lossy().to_string();

    if let Err(message) = ctx.model.set_user_input(sheet, row, col, value) {
        let message_ptr = CString::new(message)
            .expect("Couldn't create CString")
            .into_raw();

        let error = Box::into_raw(Box::new(ModelContextError {
            tag: ModelContextErrorTag::SetUserInputError,
            has_message: true,
            message: message_ptr,
        }));

        Box::into_raw(ctx) as *mut ModelContext;

        return error;
    }

    Box::into_raw(ctx) as *mut ModelContext;

    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn dispose_error(error: *mut ModelContextError) {
    let error = Box::from_raw(error);
    let str = error.message as *mut c_char;
    _ = CString::from_raw(str)
}

#[no_mangle]
pub unsafe extern "C" fn dispose(context: *mut ModelContext) {
    _ = Box::from_raw(context as *mut InternalModelContext);
}
