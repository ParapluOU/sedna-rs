#![allow(dead_code, non_camel_case_types, non_snake_case)]

use libc::{c_char, c_int};

// Constants from sp_defs.h
pub const SE_MAX_LOGIN_LENGTH: usize = 511;
pub const SE_MAX_PASSWORD_LENGTH: usize = 511;
pub const SE_MAX_DB_NAME_LENGTH: usize = 511;
pub const SE_MAX_DOCUMENT_NAME_LENGTH: usize = 511;
pub const SE_MAX_COLLECTION_NAME_LENGTH: usize = 511;
pub const SE_MAX_DIR_LENGTH: usize = 255;
pub const SE_HOSTNAMELENGTH: usize = 255;
pub const SE_SOCKET_MSG_BUF_SIZE: usize = 10240;

// Return codes from libsedna.h
pub const SEDNA_SESSION_OPEN: c_int = 1;
pub const SEDNA_SESSION_CLOSED: c_int = 2;
pub const SEDNA_AUTHENTICATION_FAILED: c_int = -3;
pub const SEDNA_QUERY_SUCCEEDED: c_int = 6;
pub const SEDNA_QUERY_FAILED: c_int = -7;
pub const SEDNA_UPDATE_SUCCEEDED: c_int = 8;
pub const SEDNA_UPDATE_FAILED: c_int = -9;
pub const SEDNA_BEGIN_TRANSACTION_SUCCEEDED: c_int = 12;
pub const SEDNA_BEGIN_TRANSACTION_FAILED: c_int = -13;
pub const SEDNA_ROLLBACK_TRANSACTION_SUCCEEDED: c_int = 14;
pub const SEDNA_ROLLBACK_TRANSACTION_FAILED: c_int = -15;
pub const SEDNA_COMMIT_TRANSACTION_SUCCEEDED: c_int = 16;
pub const SEDNA_COMMIT_TRANSACTION_FAILED: c_int = -17;
pub const SEDNA_NEXT_ITEM_SUCCEEDED: c_int = 18;
pub const SEDNA_NEXT_ITEM_FAILED: c_int = -19;
pub const SEDNA_NO_ITEM: c_int = -20;
pub const SEDNA_RESULT_END: c_int = -21;
pub const SEDNA_ERROR: c_int = -24;
pub const SEDNA_TRANSACTION_ACTIVE: c_int = 25;
pub const SEDNA_NO_TRANSACTION: c_int = 26;
pub const SEDNA_CONNECTION_OK: c_int = 27;
pub const SEDNA_CONNECTION_CLOSED: c_int = 28;
pub const SEDNA_CONNECTION_FAILED: c_int = -29;
pub const SEDNA_AUTOCOMMIT_OFF: c_int = 30;
pub const SEDNA_AUTOCOMMIT_ON: c_int = 31;
pub const SEDNA_BULK_LOAD_SUCCEEDED: c_int = 10;
pub const SEDNA_BULK_LOAD_FAILED: c_int = -11;
pub const SEDNA_DATA_CHUNK_LOADED: c_int = 23;

#[repr(C)]
pub struct conn_bulk_load {
    pub bulk_load_started: c_char,
    pub doc_name: [c_char; SE_MAX_DOCUMENT_NAME_LENGTH + 1],
    pub col_name: [c_char; SE_MAX_COLLECTION_NAME_LENGTH + 1],
}

#[repr(C)]
pub struct msg_struct {
    pub instruction: c_int,
    pub length: c_int,
    pub body: [c_char; SE_SOCKET_MSG_BUF_SIZE],
}

#[repr(C)]
pub struct SednaConnection {
    pub url: [c_char; SE_HOSTNAMELENGTH + 1],
    pub db_name: [c_char; SE_MAX_DB_NAME_LENGTH + 1],
    pub login: [c_char; SE_MAX_LOGIN_LENGTH + 1],
    pub password: [c_char; SE_MAX_PASSWORD_LENGTH + 1],
    pub session_directory: [c_char; SE_MAX_DIR_LENGTH + 1],
    pub socket: c_int,
    pub last_error: c_int,
    pub last_error_msg: [c_char; SE_SOCKET_MSG_BUF_SIZE],
    pub query_time: [c_char; 1024],
    pub socket_keeps_data: c_char,
    pub first_next: c_char,
    pub result_end: c_char,
    pub in_query: c_char,
    pub cbl: conn_bulk_load,
    pub isInTransaction: c_int,
    pub isConnectionOk: c_int,
    pub autocommit: c_char,
    pub local_data_length: c_int,
    pub local_data_offset: c_int,
    pub local_data_buf: [c_char; SE_SOCKET_MSG_BUF_SIZE],
    pub msg: msg_struct,
    pub debug_handler: *mut libc::c_void,
    pub boundary_space_preserve: c_char,
    pub cdata_preserve: c_char,
    pub query_timeout: c_int,
    pub max_result_size: c_int,
}

impl Default for SednaConnection {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// External C functions from libsedna
extern "C" {
    pub fn SEconnect(
        conn: *mut SednaConnection,
        host: *const c_char,
        db_name: *const c_char,
        login: *const c_char,
        password: *const c_char,
    ) -> c_int;

    pub fn SEclose(conn: *mut SednaConnection) -> c_int;

    pub fn SEbegin(conn: *mut SednaConnection) -> c_int;

    pub fn SErollback(conn: *mut SednaConnection) -> c_int;

    pub fn SEcommit(conn: *mut SednaConnection) -> c_int;

    pub fn SEexecute(conn: *mut SednaConnection, query: *const c_char) -> c_int;

    pub fn SEexecuteLong(conn: *mut SednaConnection, query_file_path: *const c_char) -> c_int;

    pub fn SEgetData(conn: *mut SednaConnection, buf: *mut c_char, bytes_to_read: c_int) -> c_int;

    pub fn SEloadData(
        conn: *mut SednaConnection,
        buf: *const c_char,
        bytes_to_load: c_int,
        doc_name: *const c_char,
        col_name: *const c_char,
    ) -> c_int;

    pub fn SEendLoadData(conn: *mut SednaConnection) -> c_int;

    pub fn SEnext(conn: *mut SednaConnection) -> c_int;

    pub fn SEgetLastErrorCode(conn: *mut SednaConnection) -> c_int;

    pub fn SEgetLastErrorMsg(conn: *mut SednaConnection) -> *const c_char;

    pub fn SEconnectionStatus(conn: *mut SednaConnection) -> c_int;

    pub fn SEtransactionStatus(conn: *mut SednaConnection) -> c_int;

    pub fn SEshowTime(conn: *mut SednaConnection) -> *const c_char;
}
