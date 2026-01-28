use ironcalc::base::cell;
use ironcalc::base::Model;
use std::ffi::{c_char, CString};

#[repr(C)]
pub struct ModelContext;

#[repr(C)]
pub enum ModelContextErrorTag {
    XlsxError = 1,
    WorkbookError = 2,
    SetUserInputError = 3,
    GetUserInputError = 4,
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

fn create_error(message: String, tag: ModelContextErrorTag) -> *mut ModelContextError {
    let message_ptr = CString::new(message)
        .expect("Couldn't create CString")
        .into_raw();

    Box::into_raw(Box::new(ModelContextError {
        tag,
        has_message: true,
        message: message_ptr,
    }))
}

impl CreateModelContextResult {
    fn create_error(message: String, tag: ModelContextErrorTag) -> CreateModelContextResult {
        CreateModelContextResult {
            model: std::ptr::null_mut(),
            error: create_error(message, tag),
            is_ok: false,
        }
    }
}

struct InternalModelContext {
    model: Model,
}

#[no_mangle]
pub extern "C" fn load_from_xlsx_bytes(
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

#[repr(C)]
#[derive(PartialEq)]
pub enum CellValueTag {
    None = 0,
    String = 1,
    Number = 2,
    Boolean = 3,
}

#[repr(C)]
pub struct CellValue {
    tag: CellValueTag,
    string_value: *const c_char,
    number_value: f64,
    boolean_value: bool,
}

#[repr(C)]
pub struct GetValueResult {
    value: *mut CellValue,
    error: *mut ModelContextError,
    is_ok: bool,
}

#[no_mangle]
pub unsafe extern "C" fn get_cell_value_by_index(
    context: *mut ModelContext,
    sheet: u32,
    row: i32,
    col: i32,
) -> GetValueResult {
    let ctx = Box::from_raw(context as *mut InternalModelContext);

    let value = match ctx.model.get_cell_value_by_index(sheet, row, col) {
        Ok(value) => value,
        Err(message) => {
            let error = create_error(message, ModelContextErrorTag::GetUserInputError);
            Box::into_raw(ctx) as *mut ModelContext;
            return GetValueResult {
                value: std::ptr::null_mut(),
                error,
                is_ok: false,
            };
        }
    };

    let cell_value = match value {
        cell::CellValue::Number(value) => CellValue {
            tag: CellValueTag::Number,
            string_value: Default::default(),
            number_value: value,
            boolean_value: Default::default(),
        },
        cell::CellValue::None => CellValue {
            tag: CellValueTag::None,
            string_value: Default::default(),
            number_value: Default::default(),
            boolean_value: Default::default(),
        },
        cell::CellValue::Boolean(value) => CellValue {
            tag: CellValueTag::Boolean,
            string_value: Default::default(),
            number_value: Default::default(),
            boolean_value: value,
        },
        cell::CellValue::String(value) => {
            let value_ptr = CString::new(value)
                .expect("Couldn't create CString")
                .into_raw();
            CellValue {
                tag: CellValueTag::String,
                string_value: value_ptr,
                number_value: Default::default(),
                boolean_value: Default::default(),
            }
        }
    };

    let cell_value = Box::into_raw(Box::new(cell_value));

    Box::into_raw(ctx) as *mut ModelContext;

    GetValueResult {
        value: cell_value,
        error: std::ptr::null_mut(),
        is_ok: true,
    }
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
    let value = std::ffi::CStr::from_ptr(value)
        .to_string_lossy()
        .to_string();

    if let Err(message) = ctx.model.set_user_input(sheet, row, col, value) {
        let error = create_error(message, ModelContextErrorTag::SetUserInputError);
        Box::into_raw(ctx) as *mut ModelContext;
        return error;
    }

    Box::into_raw(ctx) as *mut ModelContext;

    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn dispose_cell_value(value: *mut CellValue) {
    let value = Box::from_raw(value);
    if value.tag == CellValueTag::String {
        let str = value.string_value as *mut c_char;
        _ = CString::from_raw(str)
    }
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
